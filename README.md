# crawler

[![A11yWatch](https://circleci.com/gh/A11yWatch/crawler.svg?style=svg)](https://circleci.com/gh/A11yWatch/crawler)

Crawls websites to gather all possible pages really fast and uses gRPC.

## Getting Started

Make sure to have [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) installed or use Docker. This project requires that you start up another gRPC server on port `50051` following [proto](https://github.com/A11yWatch/crawler/blob/main/proto/website.proto) spec. We are working on adding streams instead of a post hook approach. The server sends all request into another thread and follows up on extracting the links via gRPC callbacks.

1. `cargo run` or `docker compose up`

### Docker Image

You can use the program as a docker image.

[a11ywatch/crawler](https://hub.docker.com/repository/docker/a11ywatch/crawler).

## Crate

You can use the [crate](https://crates.io/crates/website_crawler) to setup a tcp server to run on the machine.

## About

This project uses the [spider](https://github.com/madeindjs/spider) crate. We also helped in the projects development to be the fastest open-source web crawler.

## gRPC

In order to use the crawler atm you need to add the grpc client based in the proto location called website.proto. Streams support is in the making to remove the extra need for the client.

## LICENSE

Check the license file in the root of the project.
