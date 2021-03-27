/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

use dotenv;
use reqwest;

use std::collections::HashMap;

#[tokio::main]
pub async fn monitor_page(serialized: String) {
	dotenv::dotenv().ok();
	let page_url = dotenv!("CRAWL_URL").to_string();
	let mut map = HashMap::new();

	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&page_url)
		.form(&map)
		.send()
		.await
		.unwrap();
}
