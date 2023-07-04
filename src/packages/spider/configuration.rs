use compact_str::CompactString;
use std::time::Duration;

/// Structure to configure `Website` crawler
/// ```rust
/// use website_crawler::spider::website::Website;
/// let mut website: Website = Website::new("https://choosealicense.com");
/// website.configuration.blacklist_url.insert(Box::new(Vec::from(["https://choosealicense.com/licenses/".into()])));
/// website.configuration.respect_robots_txt = true;
/// website.configuration.subdomains = true;
/// website.configuration.tld = true;
/// ```
#[derive(Debug, Default)]
pub struct Configuration {
    /// Respect robots.txt file and not scrape not allowed files.
    pub respect_robots_txt: bool,
    /// Allow sub-domains.
    pub subdomains: bool,
    /// Allow all tlds for domain.
    pub tld: bool,
    /// List of pages to not crawl. [optional: regex pattern matching]
    pub blacklist_url: Option<Box<Vec<CompactString>>>,
    /// User-Agent
    pub user_agent: Option<Box<CompactString>>,
    /// Polite crawling delay in milli seconds.
    pub delay: Box<u64>,
    /// proxy to use for request [todo: make breaking API change for handling].
    pub proxy: Option<Box<CompactString>>,
    /// extend crawl with sitemap.xml
    pub sitemap: bool,
    /// Request max timeout per page
    pub request_timeout: Option<Box<Duration>>,
    /// Sitemap path for domain.
    pub sitemap_path: Option<Box<CompactString>>,
}

impl Configuration {
    /// Represents crawl configuration for a website.
    pub fn new() -> Self {
        Self {
            delay: Box::new(0),
            request_timeout: Some(Box::new(Duration::from_millis(15000))),
            ..Default::default()
        }
    }
}
