FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

ARG CRAWL_URL

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

CMD [ "./target/release/a11y-watcher" ]


