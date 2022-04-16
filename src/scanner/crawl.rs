use crate::rpc::client::monitor_page_async;
use crate::rpc::client::website::ScanParams;
use spider::website::Website;
use std::env::var;

pub async fn crawl(domain: String, user_id: u32) {
    let mut website: Website = Website::new(&domain);
    let mut pages: Vec<String> = Vec::new();

    website.configuration.respect_robots_txt = true;
    website.configuration.verbose = var("RUST_LOG").unwrap() == "true";
    website.configuration.delay = 50;
    website.crawl();

    for page in website.get_pages() {
        pages.push(page.get_url().to_string());
    }

    let web_site = ScanParams {
        pages,
        user_id,
        domain,
    };

    monitor_page_async(web_site)
        .await
        .unwrap_or_else(|e| println!("{:?}", e));
}
