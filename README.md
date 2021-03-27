# crawler

crawls websites to gather all possible urls

## Getting Started

Make sure to have [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) installed.

make sure to create a .env file and add `CRAWL_URL=http://0.0.0.0:8080/api/website-crawl`.
replace CRAWL_URL with your production endpoint to accept results. A valid endpoint to accept the hook is required for the crawler to work.

1. `curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh`
2. `cargo run`

## Docker 

you can start the service with docker by running `docker build -t crawler . && docker run -dp 8000:8000 crawler`

### compose

use the docker image 

`jeffmendez19/crawler`

## Crate 

you can install the program as create at [crate](https://crates.io/crates/website_crawler)

## API

crawl - async determine all urls in a website with a post hook

POST

http://localhost:8000/crawl

Body: { url: https://www.a11ywatch.com, id: 0 }

### ENV

CARGO_RELEASE=false //determine if prod/dev build
ROCKET_ENV=dev // determine api env
CRAWL_URL="http://api:8080/api/website-crawl-background" // endpoint to send results

## LICENSE

check the license file in the root of the project.
