mod common;

use common::*;
use rocket::http::Status;

#[test]
fn test_landing() {
	let client = test_client();
	let response = client.get("/").dispatch();
	let status = response.status();

	assert_eq!(status, Status::Ok);
}

#[test]
fn test_healthcheck() {
	let client = test_client();
	let response = client.get("/_internal_/healthcheck").dispatch();
	let status = response.status();

	assert_eq!(status, Status::Ok);
}

#[test]
fn test_404() {
	let client = test_client();
	let response = client.get("/4").dispatch();
	let status = response.status();

	assert_eq!(status, Status::NotFound);
}
