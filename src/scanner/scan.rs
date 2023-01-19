use crate::rpc::client::create_client;
use crate::rpc::client::monitor_page_complete;
use crate::rpc::client::monitor_page_start;

use crate::packages::spider::website::Website;
use crate::rpc::client::website::ScanInitParams;
use ua_generator::ua::spoof_ua;

/// crawl all pages and stream request as links are found.
pub async fn scan(
    domain: String,
    user_id: u32,
    respect_robots_txt: bool,
    agent: String,
    subdomains: bool,
    tld: bool,
    proxy: String,
    sitemap: bool
) -> Result<(), core::fmt::Error> {
    let mut client = create_client().await.unwrap();
    let mut website: Website = Website::new(&domain);

    website.configuration.respect_robots_txt = respect_robots_txt;
    website.configuration.delay = 0;
    website.configuration.subdomains = subdomains;
    website.configuration.tld = tld;
    website.configuration.proxy = proxy;
    website.configuration.sitemap = sitemap;
    website.configuration.user_agent = if !agent.is_empty() {
        &agent
    } else {
        spoof_ua()
    }
    .into();

    let rq_start = ScanInitParams { domain, user_id };
    let rc_end = rq_start.clone();

    // send scan start tracking user for following request [todo: send agent used if randomized]
    monitor_page_start(&mut client, rq_start)
        .await
        .unwrap_or_else(|e| println!("{} - crawl start failed.", e));

    website.crawl_grpc(&mut client, user_id).await;

    // send scan complete
    monitor_page_complete(&mut client, rc_end)
        .await
        .unwrap_or_else(|e| println!("{} - crawl completed failed.", e));

    Ok(())
}
