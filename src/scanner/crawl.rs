use serde_json;
use spider::website::Website;

use super::super::interface::page::Page;
use super::super::hooks::monitor::monitor_page_background;
use std::thread;
use std::env::var;
use std::time::Duration;

pub fn crawl(domain: String, user_id: u32) {
    let mut website: Website = Website::new(&domain);
    let mut pages: Vec<String> = Vec::new();

    website.configuration.respect_robots_txt = true;
    website.configuration.verbose = var("RUST_LOG").unwrap() == "true";
    website.configuration.delay = 50;
    website.configuration.concurrency = (num_cpus::get() * 4) - 1;
    website.crawl();

    for page in website.get_pages() {
        pages.push(page.get_url().to_string());
    }

    let web_site = Page {
        pages,
        user_id,
        domain,
    };

    let serialized = serde_json::to_string(&web_site).unwrap();

    // TODO: RENAME BACKGROUND
    monitor_page_background(serialized).unwrap_or_else(|e| println!("{:?}", e));

    thread::sleep(Duration::from_millis(1));
}