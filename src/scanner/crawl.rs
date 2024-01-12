use crate::rpc::client::monitor_page_async;
use crate::rpc::client::website::ScanParams;
use spider::website::Website;

/// crawl all pages and gather links sending request back once finished. Built for CI usage.
pub async fn crawl(
    domain: String,
    user_id: u32,
    respect_robots_txt: bool,
    agent: String,
    subdomains: bool,
    tld: bool,
    proxy: String,
    sitemap: bool,
    delay: u64,
) -> Result<(), core::fmt::Error> {
    let mut website: Website = Website::new(&domain);

    website.configuration.respect_robots_txt = respect_robots_txt;
    website.configuration.delay = delay;
    website.configuration.subdomains = subdomains;
    website.configuration.tld = tld;

    if !proxy.is_empty() {
        website.configuration.proxies = Some(Box::new(Vec::from([proxy.into()])));
    }

    if !agent.is_empty() {
        website.configuration.user_agent = Some(Box::new(agent.into()));
    }

    if sitemap {
        website.crawl_sitemap().await;
        website.persist_links();
        website.crawl().await;
    } else {
        website.crawl().await;
    }

    let mut pages: Vec<String> = Vec::new();

    for page in website.get_links() {
        pages.push(page.inner().to_string());
    }

    let web_site = ScanParams {
        pages,
        user_id,
        domain,
        full: true,
        html: String::from(""),
    };

    monitor_page_async(web_site)
        .await
        .unwrap_or_else(|e| println!("{:?}", e));

    Ok(())
}
