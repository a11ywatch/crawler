use crate::rpc::client::monitor_page_async;
use crate::rpc::client::website::ScanParams;
use crate::packages::spider::website::Website;
use ua_generator::ua::spoof_ua;

/// crawl all pages and gather links sending request back once finished. Built for CI usage.
pub async fn crawl(domain: &String, user_id: u32, respect_robots_txt: bool, agent: &String, subdomains: bool, tld: bool) -> Result<(), core::fmt::Error> {
    let mut website: Website = Website::new(domain);
    let mut pages: Vec<String> = Vec::new();

    website.configuration.respect_robots_txt = respect_robots_txt;
    website.configuration.delay = 18;
    website.configuration.subdomains = subdomains;
    website.configuration.tld = tld;

    website.configuration.user_agent = if !agent.is_empty() {
        agent
    } else {
        spoof_ua()
    }.into();

    website.crawl();

    for page in website.get_pages() {
        pages.push(page.get_url().to_string());
    }

    let web_site = ScanParams {
        pages,
        user_id,
        domain: domain.into(),
        full: true
    };

    monitor_page_async(web_site)
        .await
        .unwrap_or_else(|e| println!("{:?}", e));


   Ok(())
}
