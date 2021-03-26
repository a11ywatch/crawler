# crawler

crawls websites to gather all possible urls

## Getting Started

Make sure to have [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) installed.

make sure to create a .env file and add `CRAWL_URL=http://api:8080/api/website-crawl` for development and `CRAWL_URL_PROD=https://yourproductionendpoint/api/website-crawl`
replace yourproductionendpoint with your domain

1. `curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh`
2. `cargo run`

## Docker 

you can start the service with docker by running `docker build -t crawler . && docker run -dp 8000:8000 crawler`

### compose

use the docker image 

`jeffmendez19/crawler`


## Dependencies

[rust]: https://www.rust-lang.org/
[rocket]: https://rocket.rs/

## LICENSE

check the license file in the root of the project.
