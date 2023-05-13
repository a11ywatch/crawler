use crate::packages::scraper::Html;
use crate::spider::utils::fetch_page_html;
use crate::spider::website::CaseInsensitiveString;
use compact_str::CompactString;
use hashbrown::HashSet;
use reqwest::Client;
use smallvec::SmallVec;
use tokio_stream::StreamExt;
use url::Url;

/// Represent a page visited. This page contains HTML scraped with [scraper](https://crates.io/crates/scraper).
#[derive(Debug, Clone)]
pub struct Page {
    /// HTML parsed with [scraper](https://crates.io/crates/scraper) lib. The html is not stored and only used to parse links.
    pub html: Option<String>,
    /// Base absolute url for page.
    base: Url,
}

lazy_static! {
    /// include only list of resources
    static ref ONLY_RESOURCES: HashSet<CaseInsensitiveString> = {
        let mut m: HashSet<CaseInsensitiveString> = HashSet::with_capacity(14);

        m.extend([
            "html", "htm", "asp", "aspx", "php", "jps", "jpsx",
            // handle .. prefix for urls ending with an extra ending
            ".html", ".htm", ".asp", ".aspx", ".php", ".jps", ".jpsx",
        ].map(|s| s.into()));

        m
    };
}

/// get the clean domain name
pub fn domain_name(domain: &Url) -> &str {
    let b = unsafe { domain.host_str().unwrap_unchecked() };
    let b = b.split('.').collect::<Vec<&str>>();

    if b.len() > 2 {
        b[1]
    } else if b.len() == 2 {
        b[0]
    } else {
        b[b.len() - 2]
    }
}

/// convert to absolute path
#[inline]
fn convert_abs_path(base: &Url, href: &str) -> Url {
    match base.join(href) {
        Ok(mut joined) => {
            joined.set_fragment(None);
            joined
        }
        Err(_) => base.clone(),
    }
}

/// html selector for valid web pages for domain.
pub fn get_page_selectors(
    url: &str,
    subdomains: bool,
    tld: bool,
) -> Option<(CompactString, smallvec::SmallVec<[CompactString; 2]>)> {
    match Url::parse(&url) {
        Ok(host) => {
            let host_name = CompactString::from(
                match convert_abs_path(&host, Default::default()).host_str() {
                    Some(host) => host.to_ascii_lowercase(),
                    _ => Default::default(),
                },
            );

            let scheme = host.scheme();

            if tld || subdomains {
                let base = Url::parse(&url).expect("Invalid page URL");
                let dname = domain_name(&base);
                let scheme = base.scheme();

                // static html group parse
                Some((
                    dname.into(),
                    smallvec::SmallVec::from([host_name, CompactString::from(scheme)]),
                ))
            } else {
                Some((
                    CompactString::default(),
                    smallvec::SmallVec::from([host_name, CompactString::from(scheme)]),
                ))
            }
        }
        _ => None,
    }
}

/// Instantiate a new page without scraping it (used for testing purposes).
pub fn build(url: &str, html: Option<String>) -> Page {
    Page {
        html: if html.is_some() { html } else { None },
        base: Url::parse(&url).expect("Invalid page URL"),
    }
}

impl Page {
    /// Instantiate a new page and gather the html.
    pub async fn new(url: &str, client: &Client) -> Self {
        build(url, fetch_page_html(&url, &client).await)
    }

    /// URL getter for page.
    pub fn get_url(&self) -> &str {
        self.base.as_str()
    }

    /// Html getter for page.
    pub fn get_html(&self) -> &str {
        unsafe { &self.html.as_deref().unwrap_unchecked() }
    }

    /// Find the links as a stream using string resource validation
    #[inline(always)]
    pub async fn links_stream(
        &self,
        selectors: &(CompactString, SmallVec<[CompactString; 2]>),
    ) -> HashSet<CaseInsensitiveString> {
        let base_domain = &selectors.0;

        let mut map: HashSet<CaseInsensitiveString> = HashSet::new();
        let html = Box::new(Html::parse_document(self.get_html()));
        tokio::task::yield_now().await;

        let mut stream = tokio_stream::iter(html.tree);
        let parent_frags = &selectors.1; // todo: allow mix match tpt
        let parent_host = &parent_frags[0];
        let parent_host_scheme = &parent_frags[1];

        while let Some(node) = stream.next().await {
            if let Some(element) = node.as_element() {
                match element.attr("href") {
                    Some(href) => {
                        let mut abs = self.abs_path(href);
                        let mut can_process = match abs.host_str() {
                            Some(host) => parent_host.ends_with(host),
                            _ => false,
                        };

                        if can_process {
                            if abs.scheme() != parent_host_scheme.as_str() {
                                let _ = abs.set_scheme(parent_host_scheme.as_str());
                            }

                            // full url path
                            let resource_url = abs.clone();

                            // clean the resource to check if valid crawl asset
                            abs.set_query(None);

                            let clean_resource = abs.as_str();
                            let hlen = clean_resource.len();
                            // a possible resource extension
                            let hchars = &clean_resource[hlen - 5..hlen];

                            if hchars.len() > 4 {
                                if let Some(position) = hchars.find('.') {
                                    let resource_ext = &hchars[position + 1..hchars.len()];

                                    if !ONLY_RESOURCES
                                        .contains::<CaseInsensitiveString>(&resource_ext.into())
                                    {
                                        can_process = false;
                                    }
                                }
                            }

                            if can_process && base_domain.is_empty()
                                || base_domain.as_str() == domain_name(&abs)
                            {
                                map.insert(resource_url.as_str().to_string().into());
                            }
                        }
                    }
                    _ => (),
                };
            }
        }

        map
    }

    /// Find all href links and return them using CSS selectors.
    #[inline(never)]
    pub async fn links(
        &self,
        selectors: &(CompactString, SmallVec<[CompactString; 2]>),
    ) -> HashSet<CaseInsensitiveString> {
        match self.html {
            None => Default::default(),
            Some(_) => self.links_stream(&selectors).await,
        }
    }

    /// Convert a URL to its absolute path without any fragments or params.
    #[inline]
    fn abs_path(&self, href: &str) -> Url {
        convert_abs_path(&self.base, href)
    }
}

#[tokio::test]
async fn parse_links() {
    let client = Client::builder()
        .user_agent("spider/1.1.2")
        .build()
        .unwrap();

    let link_result = "https://choosealicense.com/";
    let page: Page = Page::new(&link_result, &client).await;
    let selector = get_page_selectors(&link_result, false, false);

    let links = page.links(&selector.unwrap()).await;

    assert!(
        links.contains::<CaseInsensitiveString>(&"https://choosealicense.com/about/".into()),
        "Could not find {}. Theses URLs was found {:?}",
        page.get_url(),
        &links
    );
}

#[tokio::test]
async fn test_abs_path() {
    let client = Client::builder()
        .user_agent("spider/1.1.2")
        .build()
        .unwrap();
    let link_result = "https://choosealicense.com/";
    let page: Page = Page::new(&link_result, &client).await;

    assert_eq!(
        page.abs_path("/page"),
        Url::parse("https://choosealicense.com/page").unwrap()
    );
    assert_eq!(
        page.abs_path("/page?query=keyword"),
        Url::parse("https://choosealicense.com/page?query=keyword").unwrap()
    );
    assert_eq!(
        page.abs_path("/page#hash"),
        Url::parse("https://choosealicense.com/page").unwrap()
    );
    assert_eq!(
        page.abs_path("/page?query=keyword#hash"),
        Url::parse("https://choosealicense.com/page?query=keyword").unwrap()
    );
    assert_eq!(
        page.abs_path("#hash"),
        Url::parse("https://choosealicense.com/").unwrap()
    );
    assert_eq!(
        page.abs_path("tel://+212 3456"),
        Url::parse("https://choosealicense.com/").unwrap()
    );
}
