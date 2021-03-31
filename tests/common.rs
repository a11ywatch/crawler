#![allow(unused)]
use once_cell::sync::OnceCell;
use rocket::http::{ContentType, Header, Status};
use rocket::local::{Client, LocalResponse};
use serde_json::Value;
use website_crawler;

#[macro_export]
macro_rules! json_string {
	($value:tt) => {
		serde_json::to_string(&serde_json::json!($value)).expect("cannot json stringify")
	};
}

pub fn test_client() -> &'static Client {
	static INSTANCE: OnceCell<Client> = OnceCell::new();
	INSTANCE.get_or_init(|| {
		let rocket = website_crawler::rocket();
		Client::new(rocket).expect("valid rocket instance")
	})
}

pub fn response_json_value(response: &mut LocalResponse) -> Value {
	let body = response.body().expect("no body");
	serde_json::from_reader(body.into_inner()).expect("can't parse value")
}
