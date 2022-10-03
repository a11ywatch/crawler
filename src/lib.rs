#![forbid(unsafe_code)]
extern crate tokio;
extern crate tonic;
extern crate ua_generator;

// packages mainly for spider
extern crate hashbrown;
extern crate log;
extern crate reqwest;
extern crate scraper;
extern crate url;
#[macro_use]
extern crate lazy_static;
pub use packages::spider;

#[macro_use]
extern crate string_concat;

// internal packages.
pub mod interface;
pub mod packages;
pub mod rpc;
pub mod scanner;
pub use rpc::handlers::grpc_start;
