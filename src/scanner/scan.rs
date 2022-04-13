use serde_json;
use spider::website::Website;
use std::thread;
use std::env::var;
use std::time::Duration;
use super::super::interface::page::Page;
use super::super::hooks::monitor::monitor_page_start;
use super::super::hooks::monitor::monitor_page_complete;
use crate::rpc::client::monitor_page;

pub fn scan(domain: String, user_id: u32) {
    let mut website: Website = Website::new(&domain);
    website.configuration.respect_robots_txt = true;
    website.configuration.delay = 25;
    website.configuration.verbose = var("RUST_LOG").unwrap() == "true";
    
    let web_site = Page {
        pages: [].to_vec(),
        domain,
        user_id
    };
    let web_json = &serde_json::to_string(&web_site).unwrap();

    monitor_page_start(web_json).unwrap_or_else(|e| println!("{:?}", e));

    website.on_link_find_callback = monitor_page;

    website.crawl();

    // TODO: REMOVE DURATIN FOR QUEUE
    thread::sleep(Duration::from_millis(250));

    monitor_page_complete(web_json).unwrap_or_else(|e| println!("{:?}", e))
}