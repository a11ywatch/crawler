FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV ROCKET_ENV="prod"

ARG CRAWL_URL
ARG CRAWL_URL_BACKGROUND

ARG SCAN_URL_COMPLETE
ARG SCAN_URL_START

ENV CRAWL_URL="${CRAWL_URL:-http://api:8080/api/website-crawl}"
ENV CRAWL_URL_BACKGROUND="${CRAWL_URL_BACKGROUND:-http://api:8080/api/website-crawl-background}"
ENV SCAN_URL_COMPLETE="${SCAN_URL_COMPLETE:-http://api:8080/api/website-crawl-background-complete}"
ENV SCAN_URL_START="${SCAN_URL_START:-http://api:8080/api/website-crawl-background-start}"

RUN apt-get update -y && apt-get install -y openssl libssl-dev

WORKDIR /usr/src/app


COPY . .

CMD [ "cargo", "run", "--release"]