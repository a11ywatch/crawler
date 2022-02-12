FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV ROCKET_ENV="prod"

ARG CRAWL_URL
ARG CRAWL_URL_COMPLETE

ENV CRAWL_URL="${CRAWL_URL:-http://api:8080/api/website-crawl}"
ENV CRAWL_URL_COMPLETE="${CRAWL_URL_COMPLETE:-http://api:8080/api/website-crawl-complete}"

RUN apt-get update -y && apt-get install -y openssl libssl-dev

WORKDIR /usr/src/app

RUN echo "CRAWL_URL=$CRAWL_URL" >> .env

COPY . .

CMD [ "cargo", "run", "--release"]