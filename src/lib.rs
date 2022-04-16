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
extern crate tonic;

pub mod interface;
pub mod routes;
pub mod rpc;
pub mod scanner;

pub use rpc::server::grpc_start;

pub fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![routes::status::get_health])
}
