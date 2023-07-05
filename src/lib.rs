extern crate sitemap;
extern crate tokio;
extern crate tonic;

#[cfg(feature = "ua_generator")]
extern crate ua_generator;

// packages mainly for spider
extern crate hashbrown;
extern crate log;
extern crate reqwest;
extern crate url;
#[macro_use]
extern crate lazy_static;
pub extern crate compact_str;
pub use packages::spider;
#[macro_use]
extern crate fast_html5ever;
#[macro_use]
extern crate string_concat;

// internal packages.
pub mod interface;
pub mod packages;
pub mod rpc;
pub mod scanner;
pub use rpc::handlers::grpc_start;
