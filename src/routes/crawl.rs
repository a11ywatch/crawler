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

use super::super::interface::page::Page;
use super::super::interface::website::WebPage;
use super::monitor::monitor_page;
use std::thread;
use std::time::Duration;

#[post("/crawl", format = "json", data = "<user>")]
pub fn crawl_page(user: Json<WebPage>) -> String {
	
	let handle = thread::spawn(move || {
		let domain = String::from(&user.url);
		let mut website: Website = Website::new(&domain);
		let mut pages: Vec<String> = Vec::new();
	
		website.configuration.respect_robots_txt = true;
		website.configuration.verbose = true;
		website.configuration.concurrency = num_cpus::get() | 4;
		website.crawl();
	
		for page in website.get_pages() {
			pages.push(page.get_url().to_string());
		}
	
		let web_site = Page {
			pages,
			user_id: user.id,
			domain,
		};
	
		let serialized = serde_json::to_string(&web_site).unwrap();

		monitor_page(serialized);
		thread::sleep(Duration::from_millis(1));
	});

	drop(handle);

	format!("Crawling page")
}
