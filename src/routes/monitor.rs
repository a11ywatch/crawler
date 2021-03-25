/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

// #[macro_use] extern crate dotenv_codegen;
use dotenv;
use reqwest;

use std::collections::HashMap;

#[tokio::main]
pub async fn monitor_page(serialized: String) {
	let crawl_api_url = dotenv::var("CRAWL_URL").unwrap();
	let page_url = crawl_api_url.to_string();
	let mut map = HashMap::new();

	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&page_url)
		.form(&map)
		.send()
		.await
		.unwrap();
}
