use super::black_list::contains;
use super::configuration::Configuration;
use super::page::{build, Page, get_page_selectors};
use super::robotparser::RobotFileParser;
use super::utils::log;
use crate::rpc::client::{monitor, WebsiteServiceClient};
use hashbrown::HashSet;
use reqwest::header::CONNECTION;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use scraper::Selector;
use sitemap::{
    reader::{SiteMapEntity, SiteMapReader},
    structs::Location,
};
use std::sync::Arc;
use std::time::Duration;
use tokio;
use tokio::sync::mpsc::{
    channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::task;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use url::Url;

/// Represents a website to crawl and gather all links.
/// ```rust
/// use website_crawler::spider::website::Website;
/// let mut localhost = Website::new("http://example.com");
/// localhost.crawl();
/// // `Website` will be filled with `Pages` when crawled. To get them, just use
/// for page in localhost.get_pages() {
///     // do something
/// }
/// ```
#[derive(Debug)]
pub struct Website {
    /// configuration properties for website.
    pub configuration: Configuration,
    /// the domain name of the website
    domain: String,
    /// contains all non-visited URL.
    links: HashSet<String>,
    /// contains all visited URL.
    links_visited: HashSet<String>,
    /// contains page visited
    pages: Vec<Page>,
    /// Robot.txt parser holder.
    robot_file_parser: Option<RobotFileParser>,
    /// current sitemap url
    sitemap_url: String,
}

/// hashset string_concat
pub type Message = HashSet<String>;

impl Website {
    /// Initialize Website object with a start link to crawl.
    pub fn new(domain: &str) -> Self {
        let domain_base = string_concat!(&domain, "/");

        Self {
            configuration: Configuration::new(),
            links_visited: HashSet::new(),
            links: HashSet::from([domain_base.clone()]), // todo: remove dup mem usage for domain tracking
            domain: domain_base,
            pages: Vec::new(),
            robot_file_parser: None,
            sitemap_url: String::from(""),
        }
    }

    /// page clone
    pub fn get_pages(&self) -> Vec<Page> {
        if !self.pages.is_empty() {
            self.pages.clone()
        } else {
            self.links_visited.iter().map(|l| build(l, "")).collect()
        }
    }

    /// links visited getter
    pub fn get_links(&self) -> &HashSet<String> {
        &self.links_visited
    }

    /// crawl delay getter
    fn get_delay(&self) -> Duration {
        Duration::from_millis(self.configuration.delay)
    }

    /// configure the robots parser on initial crawl attempt and run
    pub async fn configure_robots_parser(&mut self, client: &Client) {
        if self.configuration.respect_robots_txt {
            let mut robot_file_parser: RobotFileParser = match &self.robot_file_parser {
                Some(parser) => parser.to_owned(),
                _ => {
                    let mut robot_file_parser =
                        RobotFileParser::new(&string_concat!(self.domain, "robots.txt"));

                    robot_file_parser.user_agent = self.configuration.user_agent.to_owned();

                    robot_file_parser
                }
            };

            // get the latest robots todo determine time elaspe
            if robot_file_parser.mtime() <= 4000 {
                robot_file_parser.read(client).await;
                self.configuration.delay = robot_file_parser
                    .get_crawl_delay(&robot_file_parser.user_agent) // returns the crawl delay in seconds
                    .unwrap_or_else(|| self.get_delay())
                    .as_millis() as u64;
            }

            self.robot_file_parser = Some(robot_file_parser);
        }
    }

    /// configure http client
    fn configure_http_client(&mut self) -> Client {
        lazy_static! {
            static ref HEADERS: HeaderMap<HeaderValue> = {
                let mut headers = HeaderMap::new();
                headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));

                headers
            };
        }

        let default_policy = reqwest::redirect::Policy::default();
        let policy = reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.url().host_str() == Some("127.0.0.1") {
                attempt.stop()
            } else {
                default_policy.redirect(attempt)
            }
        });

        let mut client = Client::builder()
            .default_headers(HEADERS.clone())
            .redirect(policy)
            .user_agent(&self.configuration.user_agent)
            .brotli(true);

        if !self.configuration.proxy.is_empty() {
            match reqwest::Proxy::all(&self.configuration.proxy) {
                Ok(proxy) => match Url::parse(&self.configuration.proxy) {
                    Ok(url) => {
                        let password = match &url.password() {
                            Some(pass) => pass,
                            _ => "",
                        };
                        client = client.proxy(proxy.basic_auth(&url.username(), &password));
                    }
                    _ => {
                        client = client.proxy(proxy);
                    }
                },
                _ => {
                    log("proxy connect error", "");
                }
            };
        }

        client.build().unwrap_or_default()
    }

    /// setup config for crawl
    pub async fn setup(&mut self) -> Client {
        let client = self.configure_http_client();
        self.configure_robots_parser(&client).await;

        client
    }

    /// Start to crawl website with async parallelization
    pub async fn crawl(&mut self) {
        let client = self.setup().await;

        self.crawl_concurrent(&client).await;
    }

    /// Start to crawl website with gRPC streams
    pub async fn crawl_grpc(
        &mut self,
        rpc_client: &mut WebsiteServiceClient<Channel>,
        user_id: u32,
    ) {
        let client = self.setup().await;

        self.crawl_concurrent_rpc(&client, rpc_client, user_id)
            .await;
    }

    /// Start to crawl website concurrently
    async fn crawl_concurrent(&mut self, client: &Client) {
        // crawl delay between
        let delay = self.configuration.delay;
        let delay_enabled = delay > 0;
        // crawl page walking
        let subdomains = self.configuration.subdomains;
        let tld = self.configuration.tld;

        let selector = Arc::new(get_page_selectors(&self.domain, subdomains, tld));

        // crawl while links exists
        while !self.links.is_empty() {
            let (tx, mut rx): (Sender<Message>, Receiver<Message>) = channel(100);

            for link in self.links.iter() {
                if !self.is_allowed(link) {
                    continue;
                }
                log("fetch", &link);
                self.links_visited.insert(link.into());

                let tx = tx.clone();
                let client = client.clone();
                let link = link.clone();
                let selector = selector.clone();

                task::spawn(async move {
                    {
                        if delay_enabled {
                            sleep(Duration::from_millis(delay)).await;
                        }
                        let page = Page::new(&link, &client).await;
                        let links = page.links(&*selector, subdomains, tld);

                        if let Err(_) = tx.send(links).await {
                            log("receiver dropped", "");
                        }
                    }
                });
            }

            drop(tx);

            let mut new_links: HashSet<String> = HashSet::new();

            while let Some(msg) = rx.recv().await {
                new_links.extend(msg);
            }

            self.links = &new_links - &self.links_visited;
        }
    }

    /// Start to crawl website concurrently using gRPC callback
    async fn crawl_concurrent_rpc(
        &mut self,
        client: &Client,
        rpcx: &mut WebsiteServiceClient<Channel>,
        user_id: u32,
    ) {
        let subdomains = self.configuration.subdomains;
        let tld = self.configuration.tld;

        let (txx, mut rxx): (UnboundedSender<bool>, UnboundedReceiver<bool>) = unbounded_channel();
        let semaphore = Arc::new(Semaphore::new(200));
        let selector = Arc::new(get_page_selectors(&self.domain, subdomains, tld));

        // determine if crawl is still active
        let handle = task::spawn(async move {
            let mut crawl_valid = true;
            while let Some(msg) = rxx.recv().await {
                if !msg {
                    crawl_valid = false;
                    break;
                }
            }
            crawl_valid
        });

        if self.configuration.sitemap {
            self.sitemap_crawl(
                &handle, client, rpcx, &semaphore, &selector, user_id, &txx, subdomains, tld,
            )
            .await;
        };

        self.inner_crawl(
            &handle, client, rpcx, &semaphore, &selector, user_id, &txx, subdomains, tld,
        )
        .await;
    }

    /// get the entire list of urls in a sitemap
    pub async fn sitemap_crawl(
        &mut self,
        handle: &tokio::task::JoinHandle<bool>,
        client: &Client,
        rpcx: &mut WebsiteServiceClient<Channel>,
        semaphore: &Arc<Semaphore>,
        selector: &Arc<Selector>,
        user_id: u32,
        txx: &UnboundedSender<bool>,
        subdomains: bool,
        tld: bool,
    ) {
        self.sitemap_url = string_concat!(self.domain, "sitemap.xml");

        while !handle.is_finished() && !self.sitemap_url.is_empty() {
            let mut sitemap_added = false;
            let txx = txx.clone();

            match client.get(&self.sitemap_url).send().await {
                Ok(response) => {
                    match response.text().await {
                        Ok(text) => {
                            for entity in SiteMapReader::new(text.as_bytes()) {
                                if handle.is_finished() {
                                    break;
                                };
                                match entity {
                                    SiteMapEntity::Url(url_entry) => match url_entry.loc {
                                        Location::None => {}
                                        Location::Url(url) => {
                                            let link = url.as_str().into();

                                            if !self.is_allowed(&link) {
                                                continue;
                                            };

                                            self.links.insert(link);
                                        }
                                        Location::ParseErr(error) => {
                                            log("parse error entry url: ", error.to_string())
                                        }
                                    },
                                    SiteMapEntity::SiteMap(sitemap_entry) => {
                                        match sitemap_entry.loc {
                                            Location::None => {}
                                            Location::Url(url) => {
                                                self.sitemap_url = url.as_str().into();
                                                sitemap_added = true;
                                            }
                                            Location::ParseErr(err) => {
                                                log("parse error sitemap url: ", err.to_string())
                                            }
                                        }
                                    }
                                    SiteMapEntity::Err(err) => {
                                        log("incorrect sitemap error: ", err.msg())
                                    }
                                };

                                // crawl between each link
                                self.inner_crawl(
                                    &handle, client, rpcx, &semaphore, &selector, user_id, &txx,
                                    subdomains, tld,
                                )
                                .await;
                            }
                        }
                        Err(err) => log("http parse error: ", err.to_string()),
                    };
                }
                Err(err) => log("http network error: ", err.to_string()),
            };

            if !sitemap_added {
                self.sitemap_url.clear();
            };
        }
    }

    /// inner crawl pages until no links exist
    async fn inner_crawl(
        &mut self,
        handle: &tokio::task::JoinHandle<bool>,
        client: &Client,
        rpcx: &mut WebsiteServiceClient<Channel>,
        semaphore: &Arc<Semaphore>,
        selector: &Arc<Selector>,
        user_id: u32,
        txx: &UnboundedSender<bool>,
        subdomains: bool,
        tld: bool,
    ) {
        while !self.links.is_empty() && !handle.is_finished() {
            let (tx, mut rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
                unbounded_channel();
            let mut stream = tokio_stream::iter(&self.links);
            let txx = txx.clone();

            while let Some(link) = stream.next().await {
                if handle.is_finished() {
                    break;
                }
                if !self.is_allowed(&link) {
                    continue;
                }
                self.links_visited.insert(link.into());
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                log("fetch", &link);
                self.rpc_callback(
                    &rpcx, &client, &txx, &tx, &selector, link, user_id, subdomains, tld, permit,
                )
                .await;
            }

            drop(tx);
            drop(txx);

            let mut new_links: HashSet<String> = HashSet::new();

            while let Some(msg) = rx.recv().await {
                new_links.extend(msg);
                task::yield_now().await;
            }

            self.links = &new_links - &self.links_visited;
            task::yield_now().await;
        }
    }

    /// perform the rpc callback on new link
    async fn rpc_callback(
        &self,
        rpcx: &WebsiteServiceClient<Channel>,
        client: &Client,
        txx: &UnboundedSender<bool>,
        tx: &UnboundedSender<Message>,
        selector: &Arc<Selector>,
        link: &String,
        user_id: u32,
        subdomains: bool,
        tld: bool,
        permit: OwnedSemaphorePermit,
    ) {
        let mut rpcx = rpcx.clone();
        let txx = txx.clone();
        let tx = tx.clone();
        task::yield_now().await;
        let client = client.clone();
        let link = link.to_owned();
        let selector = selector.clone();

        task::yield_now().await;

        task::spawn(async move {
            {
                let page = Page::new(&link, &client).await;
                let links = page.links(&*selector, subdomains, tld);
                drop(permit);

                task::spawn(async move {
                    let x = monitor(&mut rpcx, link, user_id, page.html).await;

                    if let Err(_) = txx.send(x) {
                        log("receiver dropped", "");
                    }
                });

                if let Err(_) = tx.send(links) {
                    log("receiver dropped", "");
                }
            }
        });
    }

    /// return `true` if URL:
    ///
    /// - is not already crawled
    /// - is not blacklisted
    /// - is not forbidden in robot.txt file (if parameter is defined)  
    pub fn is_allowed(&self, link: &String) -> bool {
        if self.links_visited.contains(link) {
            return false;
        }
        if contains(&self.configuration.blacklist_url, link) {
            return false;
        }
        if self.configuration.respect_robots_txt && !self.is_allowed_robots(link) {
            return false;
        }

        true
    }

    /// return `true` if URL:
    ///
    /// - is not forbidden in robot.txt file (if parameter is defined)  
    pub fn is_allowed_robots(&self, link: &String) -> bool {
        if self.configuration.respect_robots_txt {
            let robot_file_parser = self.robot_file_parser.as_ref().unwrap(); // unwrap will always return

            robot_file_parser.can_fetch("*", link)
        } else {
            true
        }
    }
}

#[tokio::test]
async fn crawl() {
    let mut website: Website = Website::new("https://choosealicense.com");
    website.crawl().await;
    assert!(
        website
            .links_visited
            .contains(&"https://choosealicense.com/licenses/".to_string()),
        "{:?}",
        website.links_visited
    );
}

#[tokio::test]
async fn crawl_invalid() {
    let url = "https://w.com";
    let mut website: Website = Website::new(&url);
    website.crawl().await;
    let mut uniq = HashSet::new();
    uniq.insert(format!("{}/", url.to_string())); // TODO: remove trailing slash mutate

    assert_eq!(website.links_visited, uniq); // only the target url should exist
}

#[tokio::test]
async fn not_crawl_blacklist() {
    let mut website: Website = Website::new("https://choosealicense.com");
    website
        .configuration
        .blacklist_url
        .push("https://choosealicense.com/licenses/".to_string());
    website.crawl().await;
    assert!(
        !website
            .links_visited
            .contains(&"https://choosealicense.com/licenses/".to_string()),
        "{:?}",
        website.links_visited
    );
}

#[tokio::test]
async fn test_respect_robots_txt() {
    let mut website: Website = Website::new("https://stackoverflow.com");
    website.configuration.respect_robots_txt = true;
    website.configuration.user_agent = "*".into();

    let client = website.setup().await;
    website.configure_robots_parser(&client).await;

    assert_eq!(website.configuration.delay, 250);

    assert!(!website.is_allowed(&"https://stackoverflow.com/posts/".to_string()));

    // test match for bing bot
    let mut website_second: Website = Website::new("https://www.mongodb.com");
    website_second.configuration.respect_robots_txt = true;
    website_second.configuration.user_agent = "bingbot".into();

    let client_second = website_second.setup().await;
    website_second.configure_robots_parser(&client_second).await;

    assert_eq!(
        website_second.configuration.user_agent,
        website_second
            .robot_file_parser
            .as_ref()
            .unwrap()
            .user_agent
    );
    assert_eq!(website_second.configuration.delay, 60000); // should equal one minute in ms

    // test crawl delay with wildcard agent [DOES not work when using set agent]
    let mut website_third: Website = Website::new("https://www.mongodb.com");
    website_third.configuration.respect_robots_txt = true;
    let client_third = website_third.setup().await;

    website_third.configure_robots_parser(&client_third).await;

    assert_eq!(website_third.configuration.delay, 10000); // should equal 10 seconds in ms
}

#[tokio::test]
async fn test_link_duplicates() {
    fn has_unique_elements<T>(iter: T) -> bool
    where
        T: IntoIterator,
        T::Item: Eq + std::hash::Hash,
    {
        let mut uniq = HashSet::new();
        iter.into_iter().all(move |x| uniq.insert(x))
    }

    let mut website: Website = Website::new("http://0.0.0.0:8000");
    website.crawl().await;

    assert!(has_unique_elements(&website.links_visited));
}
