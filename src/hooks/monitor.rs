use reqwest;
use std::collections::HashMap;
use std::env::var;

#[tokio::main]
pub async fn monitor_page(serialized: String) -> Result<(), reqwest::Error> {
	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&var("CRAWL_URL").unwrap())
		.form(&map)
		.send()
		.await?;

	Ok(())
}

#[tokio::main]
pub async fn monitor_page_background(serialized: String) -> Result<(), reqwest::Error> {
	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&var("CRAWL_URL_BACKGROUND").unwrap())
		.form(&map)
		.send()
		.await?;

	Ok(())
}

#[tokio::main]
pub async fn monitor_page_start(serialized: &String) -> Result<(), reqwest::Error> {
	let endpoint = var("SCAN_URL_START").unwrap();

	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&endpoint)
		.form(&map)
		.send()
		.await?;

	Ok(())
}

#[tokio::main]
pub async fn monitor_page_complete(serialized: &String) -> Result<(), reqwest::Error> {
	let mut map = HashMap::new();
	map.insert("data", serialized);

	reqwest::Client::new()
		.post(&var("SCAN_URL_COMPLETE").unwrap())
		.form(&map)
		.send()
		.await?;

	Ok(())
}