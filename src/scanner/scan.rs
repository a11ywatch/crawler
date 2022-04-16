use crate::rpc::client::create_client;
use crate::rpc::client::monitor_page;
use crate::rpc::client::monitor_page_complete;
use crate::rpc::client::monitor_page_start;

use crate::rpc::client::website::ScanParams;
use spider::website::Website;
use std::env::var;

pub async fn scan(domain: String, user_id: u32) -> Result<(), core::fmt::Error> {
    let mut client = create_client().await.unwrap();
    let mut website: Website = Website::new(&domain);
    website.configuration.respect_robots_txt = true;
    website.configuration.delay = 25;
    website.configuration.verbose = var("RUST_LOG").unwrap() == "true";
    website.on_link_find_callback = monitor_page;

    let web_site = ScanParams {
        pages: [].to_vec(),
        domain,
        user_id,
    };

    monitor_page_start(&mut client, &web_site)
        .await
        .expect("crawl message start failed.");

    website.crawl();

    monitor_page_complete(&mut client, &web_site)
        .await
        .expect("crawl message completed failed.");

    Ok(())
}
