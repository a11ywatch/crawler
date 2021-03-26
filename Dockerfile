FROM rustlang/rust:nightly AS build

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:stretch AS package
COPY --from=build /usr/src/app/target/release/crawler ./
RUN apt-get update -y && apt-get install -y libssl-dev

EXPOSE 8000

ARG CRAWL_URL
ENV CRAWL_URL=${CRAWL_URL:-http://api:8080/api/website-crawl}
ENV ROCKET_ADDRESS=0.0.0.0

CMD ["./crawler"]