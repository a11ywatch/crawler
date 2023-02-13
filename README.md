# crawler

[![A11yWatch](https://circleci.com/gh/a11ywatch/crawler.svg?style=svg)](https://circleci.com/gh/a11ywatch/crawler)

A [gRPC](https://grpc.io/) web indexer turbo charged for performance.

## Getting Started

Make sure to have [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) installed or Docker.

This project requires that you start up another gRPC server on port `50051` following the [proto spec](https://github.com/a11ywatch/protobuf/blob/main/website.proto).

The user agent is spoofed on each crawl to a random agent and the indexer extends [spider](https://github.com/madeindjs/spider) as the base.

1. `cargo run` or `docker compose up`

## Installation

You can install easily with the following:

### Cargo

The [crate](https://crates.io/crates/website_crawler) is available to setup a gRPC server within rust projects.

```sh
cargo install website_crawler
```

### Docker

You can use also use the docker image at [a11ywatch/crawler](https://hub.docker.com/repository/docker/a11ywatch/crawler).

Set the `CRAWLER_IMAGE` env var to `darwin-arm64` to get the native m1 mac image.

```yml
crawler:
  container_name: crawler
  image: "a11ywatch/crawler:${CRAWLER_IMAGE:-latest}"
  ports:
    - 50055
```

### Node / Bun

We also release the package to npm [@a11ywatch/crawler](https://www.npmjs.com/package/@a11ywatch/crawler).

```sh
npm i @a11ywatch/crawler
```

After import at the top of your project to start the gRPC server or run node directly against the module.

```ts
import "@a11ywatch/crawler";
```

## Example

This is a basic example crawling a web page, add spider to your `Cargo.toml`:

```toml
[dependencies]
website_crawler = "0.7.36"
```

And then the code:

```rust,no_run
extern crate spider;

use website_crawler::website::Website;
use website_crawler::tokio;

#[tokio::main]
async fn main() {
    let url = "https://choosealicense.com";
    let mut website: Website = Website::new(&url);
    website.crawl().await;

    for page in website.get_pages() {
        println!("- {}", page.get_url());
    }
}
```

You can use `Configuration` object to configure your crawler:

```rust
// ..
let mut website: Website = Website::new("https://choosealicense.com");
website.configuration.blacklist_url.push("https://choosealicense.com/licenses/".to_string());
website.configuration.respect_robots_txt = true;
website.configuration.subdomains = true;
website.configuration.tld = false;
website.configuration.sitemap = false;
website.configuration.proxy = "http://username:password@localhost:1234";
website.configuration.delay = 0; // Defaults to 250 ms
website.configuration.user_agent = "myapp/version".to_string(); // Defaults to spider/x.y.z, where x.y.z is the library version
website.crawl().await;
```

### Dependencies

npm is required to install the protcol buffers.

In order to build `crawler` locally >= 0.5.0, you need the `protoc` Protocol Buffers compiler, along with Protocol Buffers resource files.

#### Ubuntu

proto compiler needs to be at v3 in order to compile. Ubuntu 18+ auto installs.

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y protobuf-compiler libprotobuf-dev
```

#### Alpine Linux

```sh
sudo apk add protoc protobuf-dev
```

#### macOS

Assuming [Homebrew](https://brew.sh/) is already installed. (If not, see instructions for installing Homebrew on [the Homebrew website](https://brew.sh/).)

```zsh
brew install protobuf
```

### Features

`jemalloc` - use jemalloc memory allocator (default disabled).

## About

This crawler is optimized for reduced latency and uses isolated based concurrency as it can handle over 10,000 pages within seconds.
In order to receive the links found for the crawler you need to add the [`website.proto`](./proto/website.proto) to your server.
This is required since every request spawns a thread. Isolating the context drastically improves performance (preventing shared resources / communication ).

## Help

If you need help implementing the gRPC server to receive the pages or links when found check out the [gRPC node example](https://github.com/A11yWatch/a11ywatch-core/blob/main/src/proto/website-server.ts) for a starting point .

## TODO

1. Allow gRPC server port setting or change to direct url:port combo.
1. add protoc pre-compiled binary [installation](https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os).

## LICENSE

Check the license file in the root of the project.
