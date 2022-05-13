use crate::rpc::client::monitor_page_async;
use crate::rpc::client::website::ScanParams;
use spider::website::Website;

/// crawl all pages and gather links sending request back once finished. Built for CI usage.
pub async fn crawl(domain: &String, user_id: u32, respect_robots_txt: bool, agent: &String) -> Result<(), core::fmt::Error> {
    let mut website: Website = Website::new(domain);
    let mut pages: Vec<String> = Vec::new();

    website.configuration.respect_robots_txt = respect_robots_txt;
    
    website.configuration.delay = 10;

    if !agent.is_empty() {
        website.configuration.user_agent = Box::leak(agent.to_owned().into_boxed_str());
    };

    website.crawl();

    for page in website.get_pages() {
        pages.push(page.get_url().to_string());
    }

    let web_site = ScanParams {
        pages,
        user_id,
        domain: domain.into(),
    };

    monitor_page_async(web_site)
        .await
        .unwrap_or_else(|e| println!("{:?}", e));


   Ok(())
}
