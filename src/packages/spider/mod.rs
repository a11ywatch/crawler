#![warn(missing_docs)]

//! Website crawling library that rapidly crawls all pages to
//! gather links in parallel
//!
//! Spider is multi-threaded crawler that can be configured
//! to scrape web pages. It has the ability to gather
//! tens of thousands of pages within seconds.
//!
//! # How to use Spider
//!
//! Example crawling with Spider:
//!
//! - **Concurrent** is the fastest way to start crawling a web page and
//!   typically the most efficient.
//!   - [`crawl`] is used to crawl concurrently :blocking.
//!
//! [`crawl`]: website/struct.Website.html#method.crawl
//!
//! # Basic usage
//!
//! First, you will need to add `spider` to your `Cargo.toml`.
//!
//! Next, simply add the website url in the struct of website and crawl,
//! you can also crawl sequentially.

/// Configuration structure for `Website`.
pub mod configuration;
/// A page scraped.
pub mod page;
/// Robot parser.
pub mod robotparser;
/// Application utils.
pub mod utils;
/// A website to crawl.
pub mod website;

/// Black list checking url exist.
pub mod black_list {
    /// check if link exist in blacklists.
    pub fn contains(blacklist_url: &Vec<String>, link: &String) -> bool {
        blacklist_url.contains(&link)
    }
}
