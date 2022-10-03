use super::black_list::contains;
use super::configuration::Configuration;
use super::page::{build, Page};
use super::robotparser::RobotFileParser;
use super::utils::log;
use crate::rpc::client::{monitor, WebsiteServiceClient};
use hashbrown::HashSet;
use rayon::prelude::*;
use reqwest::header::CONNECTION;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use std::time::Duration;
use tokio;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

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
    /// contains all non-visited URL.
    links: HashSet<String>,
    /// contains all visited URL.
    links_visited: HashSet<String>,
    /// contains page visited
    pages: Vec<Page>,
    /// Robot.txt parser holder.
    robot_file_parser: Option<RobotFileParser>,
}

type Message = HashSet<String>;

impl Website {
    /// Initialize Website object with a start link to crawl.
    pub fn new(domain: &str) -> Self {
        Self {
            configuration: Configuration::new(),
            links_visited: HashSet::new(),
            links: HashSet::from([string_concat::string_concat!(domain, "/")]),
            pages: Vec::new(),
            robot_file_parser: None,
        }
    }

    /// page clone
    pub fn get_pages(&self) -> Vec<Page> {
        if !self.pages.is_empty() {
            self.pages.clone()
        } else {
            self.links_visited
                .par_iter()
                .map(|l| build(l, ""))
                .collect()
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
                    let mut domain = String::from("");
                    // the first link upon initial config is always the domain
                    for links in self.links.iter() {
                        domain = links.clone();
                    }

                    let mut robot_file_parser =
                        RobotFileParser::new(&string_concat::string_concat!(domain, "robots.txt"));

                    robot_file_parser.user_agent = self.configuration.user_agent.to_owned();

                    robot_file_parser
                }
            };

            // get the latest robots todo determine time elaspe
            if robot_file_parser.mtime() <= 4000 {
                robot_file_parser.read(client).await;
                self.configuration.delay = robot_file_parser
                    .get_crawl_delay(&robot_file_parser.user_agent) // returns the crawl delay in seconds
                    .unwrap_or(self.get_delay())
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
        Client::builder()
            .default_headers(HEADERS.clone())
            .user_agent(&self.configuration.user_agent)
            .brotli(true)
            .build()
            .unwrap_or_default()
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

    /// Start to crawl website concurrently using gRPC callback
    async fn crawl_concurrent(&mut self, client: &Client) {
        // crawl delay between
        let delay = self.configuration.delay;
        let delay_enabled = delay > 0;
        // crawl page walking
        let subdomains = self.configuration.subdomains;
        let tld = self.configuration.tld;

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

                task::spawn(async move {
                    {
                        if delay_enabled {
                            sleep(Duration::from_millis(delay)).await;
                        }
                        let page = Page::new(&link, &client).await;
                        let links = page.links(subdomains, tld);

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
        // crawl page walking
        let subdomains = self.configuration.subdomains;
        let tld = self.configuration.tld;

        // crawl while links exists
        while !self.links.is_empty() {
            let (tx, mut rx): (Sender<Message>, Receiver<Message>) = channel(100);
            let (txx, mut rxx): (Sender<bool>, Receiver<bool>) = channel(50); // determine often

            let mut stream = tokio_stream::iter(&self.links);

            let mut crawl_valid = true; // crawl activity status

            while let Some(link) = stream.next().await {
                if !crawl_valid {
                    break;
                }
                if !self.is_allowed(&link) {
                    continue;
                }
                log("fetch", &link);
                self.links_visited.insert(link.into());

                // first spawn
                let mut rpcx = rpcx.clone();
                let l = link.clone();
                let txx = txx.clone();

                // second spawn
                let tx = tx.clone();
                let client = client.clone();
                let link = link.clone();

                task::spawn(async move {
                    {
                        let x = monitor(&mut rpcx, &l, user_id).await;

                        if let Err(_) = txx.send(x).await {
                            log("receiver dropped", "");
                        }
                    }
                });

                task::spawn(async move {
                    {
                        let page = Page::new(&link, &client).await;
                        let links = page.links(subdomains, tld);
                        task::yield_now().await;

                        if let Err(_) = tx.send(links).await {
                            log("receiver dropped", "");
                        }
                    }
                });
            }

            drop(tx);
            drop(txx);

            while let Some(msg) = rxx.recv().await {
                if !msg {
                    crawl_valid = false;
                    break;
                }
            }

            if crawl_valid {
                let mut new_links: HashSet<String> = HashSet::new();

                while let Some(msg) = rx.recv().await {
                    new_links.par_extend(msg);
                    task::yield_now().await;
                }

                self.links = &new_links - &self.links_visited;
            } else {
                self.links.clear();
            }

            task::yield_now().await;
        }
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
