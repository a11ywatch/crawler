FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV ROCKET_ENV=prod

ARG CRAWL_URL
ENV CRAWL_URL="${CRAWL_URL:-http://api:8080/api/website-crawl}"
ENV ROCKET_ENV="dev"
RUN apt-get update -y && apt-get install -y openssl libssl-dev

WORKDIR /usr/src/app

RUN echo "CRAWL_URL=$CRAWL_URL" >> .env

COPY . .

CMD [ "ROCKET_ENV=prod", "cargo", "run", "--release"]