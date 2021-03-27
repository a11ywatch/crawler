/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

use reqwest;
use std::collections::HashMap;
use std::env;

#[tokio::main]
pub async fn monitor_page(serialized: String) {
	let page_url = match env::var("CRAWL_URL") {
		Ok(val) => val.to_string(),
		Err(_e) => "http://api:8080/api/website-crawl-background".to_string(),
	};
	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&page_url)
		.form(&map)
		.send()
		.await
		.unwrap();
}
