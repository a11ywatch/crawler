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
	let endpoint = match env::var("CRAWL_URL") {
		Ok(val) => val.to_string(),
		Err(_) => "".to_string(),
	};

	if endpoint.chars().count() > 1 {
		let mut map = HashMap::new();
		map.insert("data", serialized);

		reqwest::Client::new()
			.post(&endpoint)
			.form(&map)
			.send()
			.await
			.unwrap();
	}
}

#[tokio::main]
pub async fn monitor_page_complete(serialized: String) {
	let endpoint = match env::var("CRAWL_URL_COMPLETE") {
		Ok(val) => val.to_string(),
		Err(_) => "".to_string(),
	};

	if endpoint.chars().count() > 1 {
		let mut map = HashMap::new();
		map.insert("data", serialized);

		reqwest::Client::new()
			.post(&endpoint)
			.form(&map)
			.send()
			.await
			.unwrap();
	}
}