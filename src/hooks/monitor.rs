use reqwest;
use std::collections::HashMap;
use std::env::var;

#[tokio::main]
pub async fn monitor_page(serialized: String) {
	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&var("CRAWL_URL").unwrap())
		.form(&map)
		.send()
		.await
		.unwrap();
}

#[tokio::main]
pub async fn monitor_page_background(serialized: String) {
	let endpoint = var("CRAWL_URL_BACKGROUND").unwrap();

	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&endpoint)
		.form(&map)
		.send()
		.await
		.unwrap();
}

#[tokio::main]
pub async fn monitor_page_start(serialized: String) {
	let endpoint = var("SCAN_URL_START").unwrap();

	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&endpoint)
		.form(&map)
		.send()
		.await
		.unwrap();
}

#[tokio::main]
pub async fn monitor_page_complete(serialized: String) {
	let endpoint = var("SCAN_URL_COMPLETE").unwrap();

	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&endpoint)
		.form(&map)
		.send()
		.await
		.unwrap();
}