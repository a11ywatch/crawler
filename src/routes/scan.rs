/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

 use rocket;
 use rocket_contrib;
 
 use serde_json;
 use spider;
 use num_cpus;

 use rocket_contrib::json::Json;
 use spider::website::Website;
 
 use super::super::interface::page::PageSingle;
 use super::super::interface::page::Page;
 use super::super::interface::website::WebPage;
 use super::super::hooks::monitor::monitor_page;
 use super::super::hooks::monitor::monitor_page_start;
 use super::super::hooks::monitor::monitor_page_complete;
 use std::thread;
 use std::env::var;
 use std::time::Duration;
 
 #[post("/scan", format = "json", data = "<user>")]
 pub fn scan_page(user: Json<WebPage>) -> String {

     let handle = thread::spawn(move || {
        let domain = String::from(&user.url);
        let mut website: Website = Website::new(&domain);
     
        let web_site = Page {
            pages: [].to_vec(),
            domain,
            user_id: user.id
        };
        
        monitor_page_start(serde_json::to_string(&web_site).unwrap());
        thread::sleep(Duration::from_millis(1));

        website.configuration.respect_robots_txt = true;
        website.configuration.delay = 50;
        website.configuration.verbose = var("RUST_LOG").unwrap() == "true";
        website.configuration.concurrency = (num_cpus::get() * 4) - 2;

        website.on_link_find_callback = |link| {
            let page = PageSingle {
                pages: [link.to_string()].to_vec()
            };
            monitor_page(serde_json::to_string(&page).unwrap());
            link
        };

        website.crawl();
        thread::sleep(Duration::from_millis(100));
        monitor_page_complete(serde_json::to_string(&web_site).unwrap());
     });
 
     drop(handle);
     format!("Scanning page")
 }
 