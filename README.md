# crawler

[![A11yWatch](https://circleci.com/gh/A11yWatch/crawler.svg?style=svg)](https://circleci.com/gh/A11yWatch/crawler)

crawls websites to gather all possible pages

## Getting Started

Make sure to have [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) installed.

1. `cargo run`

or

1. `docker-compose up`

### image

You can use the program as a docker image.

[a11ywatch/crawler](https://hub.docker.com/repository/docker/a11ywatch/crawler).

## Crate

you can use the [crate](https://crates.io/crates/website_crawler) to setup a tcp server to run on the machine.

## API

`crawl` - async determine all urls in a website with a post hook

```
curl --location --request POST 'http://0.0.0.0:8000/crawl' \
--header 'Content-Type: application/json' \
--data-raw '{"url": "http://www.drake.com", "id": 0 }'

// results
{
    "pages": [
        "http://www.drake.com/",
        "http://www.drake.com/?hsLang=en"
    ],
    "user_id": 0,
    "domain": "http://www.drake.com"
}
```

### ENV

```
CRAWL_URL="http://api:8080/api/website-crawl-background"
```

## LICENSE

check the license file in the root of the project.
