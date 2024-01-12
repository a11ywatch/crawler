use crate::rpc::client::create_client;
use crate::rpc::client::monitor;
use crate::rpc::client::monitor_page_complete;
use crate::rpc::client::monitor_page_start;

use crate::rpc::client::website::ScanInitParams;
use spider::utils::shutdown;
use spider::website::Website;

/// crawl all pages and stream request as links are found.
pub async fn scan(
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
    let mut client = create_client().await.unwrap();
    let mut website: Website = Website::new(&domain);

    website.configuration.respect_robots_txt = respect_robots_txt;
    website.configuration.delay = delay;
    website.configuration.subdomains = subdomains;
    website.configuration.tld = tld;

    if !proxy.is_empty() {
        website.configuration.proxies = Some(Box::new(Vec::from([proxy.into()])));
    };

    if !agent.is_empty() {
        website.configuration.user_agent = Some(Box::new(agent.into()));
    };

    website.crawl_id = Box::new(user_id.to_string());

    let rq_start = ScanInitParams { domain, user_id };
    let rc_end = rq_start.clone();

    // send scan start tracking user for following request [todo: send agent used if randomized]
    monitor_page_start(&mut client, rq_start)
        .await
        .unwrap_or_else(|e| println!("{} - crawl start failed.", e));

    let mut rx2 = website.subscribe(888).unwrap();

    tokio::spawn(async move {
        while let Ok(res) = rx2.recv().await {
            let x = monitor(
                &mut client,
                res.get_url().to_string(),
                user_id,
                res.get_html(),
            )
            .await;

            if x {
                shutdown(&string_concat!(rc_end.domain, user_id.to_string())).await;
                break;
            }
        }
        // send scan complete
        monitor_page_complete(&mut client, rc_end)
            .await
            .unwrap_or_else(|e| println!("{} - crawl completed failed.", e));
    });

    if sitemap {
        website.crawl_sitemap().await;
        website.persist_links();
        website.crawl().await;
    } else {
        website.crawl().await;
    }

    Ok(())
}
