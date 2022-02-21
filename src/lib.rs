/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

extern crate dotenv;
extern crate num_cpus;
extern crate reqwest;
extern crate serde_json;
extern crate spider;
extern crate sysinfo;

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
				routes::index::landing,
				routes::status::get_health,
				routes::status::get_cpu,
				routes::status::get_server_load,
				routes::crawl::crawl_page,
				routes::scan::scan_page,
			],
		)
		.register(catchers![not_found])
}
