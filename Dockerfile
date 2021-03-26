FROM rustlang/rust:nightly AS build

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:stretch AS package
COPY --from=build /usr/src/app/target/release/crawler ./

EXPOSE 8000
ARG CRAWL_URL
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
CMD ["./crawler"]