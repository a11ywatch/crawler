use crate::rpc::client::create_client;
use crate::rpc::client::monitor_page;
use crate::rpc::client::monitor_page_complete;
use crate::rpc::client::monitor_page_start;

use crate::rpc::client::website::ScanParams;
use spider::website::Website;

/// crawl all pages and send request as links are found. TODO: move to stream instead of callback.
pub async fn scan(domain: &String, user_id: u32, respect_robots_txt: bool, agent: &String) -> Result<(), core::fmt::Error> {
    let mut client = create_client().await.unwrap();
    let mut website: Website = Website::new(domain);

    website.configuration.respect_robots_txt = respect_robots_txt;

    website.configuration.delay = 15;
    // TODO: re-use client in monitor.
    website.on_link_find_callback = monitor_page;

    if !agent.is_empty() {
        website.configuration.user_agent = agent.into();
    };

    let web_site = ScanParams {
        pages: [].to_vec(),
        domain: domain.into(),
        user_id,
        full: false
    };

    // send scan start tracking user for following request
    monitor_page_start(&mut client, &web_site)
        .await
        .expect("crawl message start failed.");

    website.crawl();

    // send scan complete
    monitor_page_complete(&mut client, &web_site)
        .await
        .expect("crawl message completed failed.");

    Ok(())
}
