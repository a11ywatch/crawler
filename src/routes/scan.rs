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
 use super::super::hooks::monitor::monitor_page_complete;
 use std::thread;
 use std::time::Duration;
 
 #[post("/scan", format = "json", data = "<user>")]
 pub fn scan_page(user: Json<WebPage>) -> String {
     
     let handle = thread::spawn(move || {
         let domain = String::from(&user.url);
         let mut website: Website = Website::new(&domain);
     
         website.configuration.respect_robots_txt = true;
         website.configuration.verbose = true;
         website.configuration.concurrency = num_cpus::get() | 4;

         website.on_link_find_callback = |page| {
            let website_page = PageSingle {
                pages: [page.to_string()].to_vec()
            };
            let serialized = serde_json::to_string(&website_page).unwrap();
            monitor_page(serialized);
            thread::sleep(Duration::from_millis(1));
            page
         };

         website.crawl();

         let web_site = Page {
            pages: [].to_vec(),
            domain,
            user_id: user.id
        };

        let serialized = serde_json::to_string(&web_site).unwrap();
        monitor_page_complete(serialized);
        thread::sleep(Duration::from_millis(1));
     });
 
     drop(handle);
     format!("Scanning page")
 }
 