use super::black_list::contains;
use super::configuration::Configuration;
use super::page::{build, get_page_selectors, Page};
use super::robotparser::RobotFileParser;
use super::utils::log;
use crate::rpc::client::{monitor, WebsiteServiceClient};
use compact_str::CompactString;
use hashbrown::HashSet;
use reqwest::header::CONNECTION;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use scraper::Selector;
use sitemap::{
    reader::{SiteMapEntity, SiteMapReader},
    structs::Location,
};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Semaphore;
use tokio::task;
use tokio::task::JoinSet;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use ua_generator::ua::spoof_ua;
use url::Url;

/// case-insensitive string handling
#[derive(Debug, Clone)]
pub struct CaseInsensitiveString(CompactString);

impl PartialEq for CaseInsensitiveString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for CaseInsensitiveString {}

impl Hash for CaseInsensitiveString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        for c in self.0.as_bytes() {
            c.to_ascii_lowercase().hash(state)
        }
    }
}

impl From<&str> for CaseInsensitiveString {
    #[inline]
    fn from(s: &str) -> Self {
        CaseInsensitiveString { 0: s.into() }
    }
}

impl From<String> for CaseInsensitiveString {
    fn from(s: String) -> Self {
        CaseInsensitiveString { 0: s.into() }
    }
}

impl AsRef<str> for CaseInsensitiveString {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

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
    pub configuration: Box<Configuration>,
    /// the domain name of the website
    domain: Box<CompactString>,
    /// contains all non-visited URL.
    links: HashSet<CaseInsensitiveString>,
    /// contains all visited URL.
    links_visited: Box<HashSet<CaseInsensitiveString>>,
    /// Robot.txt parser holder.
    robot_file_parser: Option<Box<RobotFileParser>>,
    /// current sitemap url
    sitemap_url: Option<Box<CompactString>>,
}

/// hashset string_concat
pub type Message = HashSet<CaseInsensitiveString>;

impl Website {
    /// Initialize Website object with a start link to crawl.
    pub fn new(domain: &str) -> Self {
        let domain_base = string_concat!(&domain, "/");

        Self {
            configuration: Box::new(Configuration::new()),
            links_visited: HashSet::new().into(),
            links: HashSet::from([domain_base.clone().into()]), // todo: remove dup mem usage for domain tracking
            domain: Box::new(CompactString::new(domain_base)),
            robot_file_parser: None,
            sitemap_url: Default::default(),
        }
    }

    /// page clone
    pub fn get_pages(&self) -> Vec<Page> {
        self.links_visited
            .iter()
            .map(|l| build(&l.0, Default::default()))
            .collect()
    }

    /// links visited getter
    pub fn get_links(&self) -> &HashSet<CaseInsensitiveString> {
        &self.links_visited
    }

    /// crawl delay getter
    fn get_delay(&self) -> Duration {
        Duration::from_millis(self.configuration.delay)
    }

    /// configure the robots parser on initial crawl attempt and run
    pub async fn configure_robots_parser(&mut self, client: &Client) {
        if self.configuration.respect_robots_txt {
            let robot_file_parser = self
                .robot_file_parser
                .get_or_insert_with(|| RobotFileParser::new());

            if robot_file_parser.mtime() <= 4000 {
                robot_file_parser.read(&client, &self.domain).await;
                self.configuration.delay = robot_file_parser
                    .get_crawl_delay(&self.configuration.user_agent) // returns the crawl delay in seconds
                    .unwrap_or_else(|| self.get_delay())
                    .as_millis() as u64;
            }
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
            .tcp_keepalive(Duration::from_millis(500))
            .pool_idle_timeout(None)
            .user_agent(match &self.configuration.user_agent {
                Some(ua) => ua.as_str(),
                _ => spoof_ua(),
            })
            .brotli(true);

        match &self.configuration.proxy {
            Some(proxy_url) => {
                match reqwest::Proxy::all(proxy_url.as_str()) {
                    Ok(proxy) => match Url::parse(&*proxy_url.as_str()) {
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
            _ => {}
        }

        match &self.configuration.request_timeout {
            Some(t) => client.timeout(**t),
            _ => client,
        }
        .build()
        .unwrap_or_default()
    }

    /// setup config for crawl
    pub async fn setup(&mut self) -> Client {
        let client = self.configure_http_client();
        self.configure_robots_parser(&client).await;

        client
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

    /// Start to crawl website with async conccurency
    pub async fn crawl(&mut self) {
        let client = self.setup().await;

        self.crawl_concurrent(&client).await;
    }

    /// Start to crawl website concurrently
    async fn crawl_concurrent(&mut self, client: &Client) {
        let selector = Arc::new(get_page_selectors(
            &self.domain,
            self.configuration.subdomains,
            self.configuration.subdomains,
        ));
        let throttle = self.get_delay();
        let mut new_links: HashSet<CaseInsensitiveString> = HashSet::new();

        // crawl while links exists
        while !self.links.is_empty() {
            let (tx, mut rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
                unbounded_channel();

            let stream = tokio_stream::iter(&self.links).throttle(throttle);
            tokio::pin!(stream);

            while let Some(l) = stream.next().await {
                if !self.is_allowed(l) {
                    continue;
                }
                log("fetch", &l);
                let link = l.clone();
                self.links_visited.insert(l.to_owned());

                let tx = tx.clone();
                let client = client.clone();
                let selector = selector.clone();

                task::spawn(async move {
                    {
                        let page = Page::new(&link.0, &client).await;
                        let links = page.links(&*selector);

                        if let Err(_) = tx.send(links) {
                            log("receiver dropped", "");
                        }
                    }
                });
            }

            drop(tx);

            while let Some(msg) = rx.recv().await {
                new_links.extend(msg);
            }

            self.links.clone_from(&(&new_links - &self.links_visited));
            new_links.clear();
            if new_links.capacity() > 100 {
                new_links.shrink_to_fit()
            }
        }
    }

    /// Start to crawl website concurrently using gRPC callback
    async fn crawl_concurrent_rpc(
        &mut self,
        client: &Client,
        rpcx: &mut WebsiteServiceClient<Channel>,
        user_id: u32,
    ) {
        let (txx, mut rxx): (UnboundedSender<bool>, UnboundedReceiver<bool>) = unbounded_channel();
        let semaphore = Arc::new(Semaphore::new(200));
        let selector = Arc::new(get_page_selectors(
            &self.domain,
            self.configuration.subdomains,
            self.configuration.tld,
        ));
        let rpcx = Arc::new(rpcx);
        let throttle = self.get_delay();

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

        task::yield_now().await;

        if self.configuration.sitemap {
            self.sitemap_crawl(&handle, client, &rpcx, &semaphore, user_id, &txx)
                .await;
        };

        self.inner_crawl(
            &handle, client, &rpcx, &semaphore, &selector, &throttle, user_id, &txx,
        )
        .await;
    }

    /// inner crawl pages until no links exist
    async fn inner_crawl(
        &mut self,
        handle: &tokio::task::JoinHandle<bool>,
        client: &Client,
        rpcx: &Arc<&mut WebsiteServiceClient<Channel>>,
        semaphore: &Arc<Semaphore>,
        selector: &Arc<(Selector, CompactString)>,
        throttle: &Duration,
        user_id: u32,
        txx: &UnboundedSender<bool>,
    ) {
        let mut set: JoinSet<HashSet<CaseInsensitiveString>> = JoinSet::new();
        let mut links: HashSet<CaseInsensitiveString> =
            if self.is_allowed_default(&CompactString::new(&self.domain.as_str())) {
                let page = Page::new(&self.domain, &client).await;
                self.links_visited.insert(page.get_url().to_owned().into());

                self.rpc_callback(
                    &rpcx,
                    &client,
                    &txx,
                    &mut set,
                    &selector,
                    &self.domain,
                    user_id,
                    &semaphore,
                )
                .await;

                let mut cu: HashSet<CaseInsensitiveString> = self.links.drain().collect();

                cu.extend(page.links(&selector));

                cu
            } else {
                self.links.drain().collect()
            };

        loop {
            let stream =
                tokio_stream::iter::<HashSet<CaseInsensitiveString>>(links.drain().collect())
                    .throttle(*throttle);
            tokio::pin!(stream);

            // clone the unbounded sender
            let txx = txx.clone();

            while let Some(link) = stream.next().await {
                if handle.is_finished() {
                    break;
                }
                if !self.is_allowed(&link) {
                    continue;
                }
                self.links_visited.insert(link.clone());
                let permit = semaphore.clone();
                log("fetch", &link);
                self.rpc_callback(
                    &rpcx, &client, &txx, &mut set, &selector, &link.0, user_id, &permit,
                )
                .await;
            }
            drop(txx);

            if links.capacity() >= 1500 {
                links.shrink_to_fit();
            }

            while let Some(res) = set.join_next().await {
                let msg = res.unwrap();
                links.extend(&msg - &self.links_visited);
                task::yield_now().await;
            }

            if links.is_empty() || handle.is_finished() {
                break;
            }
        }
    }

    /// get the entire list of urls in a sitemap
    pub async fn sitemap_crawl(
        &mut self,
        handle: &tokio::task::JoinHandle<bool>,
        client: &Client,
        rpcx: &Arc<&mut WebsiteServiceClient<Channel>>,
        semaphore: &Arc<Semaphore>,
        user_id: u32,
        txx: &UnboundedSender<bool>,
    ) {
        self.sitemap_url = Some(Box::new(
            string_concat!(self.domain.as_str(), "sitemap.xml").into(),
        ));

        while !handle.is_finished() && !self.sitemap_url.is_none() {
            let mut sitemap_added = false;
            let txx = txx.clone();

            match client
                .get(self.sitemap_url.as_ref().unwrap().as_str())
                .send()
                .await
            {
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

                                            self.rpc_callback_raw(
                                                &rpcx, &client, &txx, &link.0, user_id, semaphore,
                                            )
                                            .await;

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
                                                self.sitemap_url
                                                    .replace(Box::new(url.as_str().into()));

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
                            }
                        }
                        Err(err) => log("http parse error: ", err.to_string()),
                    };
                }
                Err(err) => log("http network error: ", err.to_string()),
            };

            if !sitemap_added {
                self.sitemap_url = None;
            };
        }
    }

    /// perform the rpc callback on new link
    async fn rpc_callback(
        &self,
        rpcx: &WebsiteServiceClient<Channel>,
        client: &Client,
        txx: &UnboundedSender<bool>,
        set: &mut JoinSet<HashSet<CaseInsensitiveString>>,
        selector: &Arc<(Selector, CompactString)>,
        link: &CompactString,
        user_id: u32,
        semaphore: &Arc<Semaphore>,
    ) {
        let mut rpcx = rpcx.clone();
        let txx = txx.clone();
        task::yield_now().await;
        let client = client.clone();
        let link = link.to_string();
        let selector = selector.clone();
        let permit = semaphore.clone();

        task::yield_now().await;

        set.spawn(async move {
            let permit = permit.acquire().await.unwrap();
            let page = Page::new(&link, &client).await;
            let links = page.links(&selector);

            let x = monitor(&mut rpcx, link, user_id, page.html).await;
            drop(permit);

            if let Err(_) = txx.send(x) {
                log("receiver dropped", "");
            };

            links
        });

        task::yield_now().await;
    }

    /// perform the rpc callback raw without links
    async fn rpc_callback_raw(
        &self,
        rpcx: &WebsiteServiceClient<Channel>,
        client: &Client,
        txx: &UnboundedSender<bool>,
        link: &CompactString,
        user_id: u32,
        semaphore: &Arc<Semaphore>,
    ) {
        let mut rpcx = rpcx.clone();
        let txx = txx.clone();
        task::yield_now().await;
        let client = client.clone();
        let link = link.to_string();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        task::yield_now().await;

        task::spawn(async move {
            {
                let page = Page::new(&link, &client).await;
                let x = monitor(&mut rpcx, link, user_id, page.html).await;
                drop(permit);

                if let Err(_) = txx.send(x) {
                    log("receiver dropped", "");
                }
            }
            task::yield_now().await;
        });
    }

    /// return `true` if URL:
    ///
    /// - is not already crawled
    /// - is not blacklisted
    /// - is not forbidden in robot.txt file (if parameter is defined)
    pub fn is_allowed(&self, link: &CaseInsensitiveString) -> bool {
        if self.links_visited.contains(link) {
            false
        } else {
            self.is_allowed_default(&link.0)
        }
    }

    /// return `true` if URL:
    ///
    /// - is not blacklisted
    /// - is not forbidden in robot.txt file (if parameter is defined)
    pub fn is_allowed_default(&self, link: &CompactString) -> bool {
        if !self.configuration.blacklist_url.is_none() {
            match &self.configuration.blacklist_url {
                Some(v) => !contains(v, &link),
                _ => true,
            }
        } else {
            self.is_allowed_robots(&link)
        }
    }

    /// return `true` if URL:
    ///
    /// - is not forbidden in robot.txt file (if parameter is defined)
    pub fn is_allowed_robots(&self, link: &str) -> bool {
        if self.configuration.respect_robots_txt {
            let robot_file_parser = self.robot_file_parser.as_ref().unwrap(); // unwrap will always return

            robot_file_parser.can_fetch("*", &link)
        } else {
            true
        }
    }
}

#[tokio::test]
async fn test_respect_robots_txt() {
    let mut website: Website = Website::new("https://stackoverflow.com");
    website.configuration.respect_robots_txt = true;
    website.configuration.user_agent = Some(Box::new("*".into()));

    let client = website.setup().await;
    website.configure_robots_parser(&client).await;

    assert_eq!(website.configuration.delay, 250);

    assert!(!website.is_allowed(&"https://stackoverflow.com/posts/".into()));

    // test match for bing bot
    let mut website_second: Website = Website::new("https://www.mongodb.com");
    website_second.configuration.respect_robots_txt = true;
    website_second.configuration.user_agent = Some(Box::new("bingbot".into()));

    let client_second = website_second.setup().await;
    website_second.configure_robots_parser(&client_second).await;

    assert_eq!(website_second.configuration.delay, 60000); // should equal one minute in ms

    // test crawl delay with wildcard agent [DOES not work when using set agent]
    let mut website_third: Website = Website::new("https://www.mongodb.com");
    website_third.configuration.respect_robots_txt = true;
    let client_third = website_third.setup().await;

    website_third.configure_robots_parser(&client_third).await;

    assert_eq!(website_third.configuration.delay, 10000); // should equal 10 seconds in ms
}
