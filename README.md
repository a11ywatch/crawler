# crawler

[![A11yWatch](https://circleci.com/gh/A11yWatch/crawler.svg?style=svg)](https://circleci.com/gh/A11yWatch/crawler)

crawls websites to gather all possible pages really fast

## Getting Started

Make sure to have [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) installed.

1. `cargo run` or `docker compose up`

### image

You can use the program as a docker image.

[a11ywatch/crawler](https://hub.docker.com/repository/docker/a11ywatch/crawler).

## Crate

you can use the [crate](https://crates.io/crates/website_crawler) to setup a tcp server to run on the machine.

## gRPC

In order to use the crawler atm you need to add the grpc client based in the proto location called website.proto. Streams support is in the making to remove the extra need for the client.

## LICENSE

check the license file in the root of the project.
