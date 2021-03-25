FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

ARG CRAWL_URL

WORKDIR /usr/src/app

COPY . .

# ignore cargo build step due to run auto build
RUN cargo build --release

CMD [ "./target/release/a11y-watcher"]


