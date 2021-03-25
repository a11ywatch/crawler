/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

use rocket;
use rocket_contrib;
// #[macro_use] extern crate dotenv_codegen;

use serde_json;
use spider;

use rocket_contrib::json::Json;
use spider::website::Website;

use super::super::interface::page::Page;
use super::super::interface::website::WebPage;
use super::monitor::monitor_page;

#[post("/crawl", format = "json", data = "<user>")]
pub fn crawl_page(user: Json<WebPage>) -> String {
	// UNCOMMENT IF CAPABLE OF USING ENV VAR PRE+Build
	// if cfg!(debug_assertions) && crawl_api_url.is_empty() {
	//     crawl_api_url = dotenv!("CRAWL_URL").to_string();
	// }
	let domain = String::from(&user.url);
	let domain_clone = domain.clone();
	let mut website: Website = Website::new(&domain);
	let mut pages: Vec<String> = Vec::new();

	website.configuration.respect_robots_txt = true;
	website.configuration.verbose = true;
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

	format!("crawled {}", domain_clone)
}
