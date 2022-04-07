#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde;

extern crate num_cpus;
extern crate reqwest;
extern crate serde_json;
extern crate spider;

pub mod interface;
pub mod routes;
pub mod hooks;

use rocket_contrib::json::JsonValue;

#[catch(404)]
fn not_found() -> JsonValue {
	json!({
		"status": "error",
		"reason": "Resource was not found."
	})
}

pub fn rocket() -> rocket::Rocket {
	rocket::ignite()
		.mount(
			"/",
			routes![
				routes::status::get_health,
				routes::crawl::crawl_page,
				routes::scan::scan_page,
			],
		)
		.register(catchers![not_found])
}
