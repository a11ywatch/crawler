FROM rust:1.40 AS builder

WORKDIR /app
COPY . .

ENV GRPC_HOST=0.0.0.0:50055
ENV GRPC_HOST_API=api:50051

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential \
    gcc cmake libc6 openssl

RUN cargo install --no-default-features --path .

FROM debian:buster-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential \
    ca-certificates openssl

COPY --from=builder /usr/local/cargo/bin/website_crawler /usr/local/bin/website_crawler
COPY --from=builder /usr/local/cargo/bin/health_client /usr/local/bin/health_client

ENV GRPC_HOST=0.0.0.0:50055
ENV GRPC_HOST_API=api:50051

CMD ["website_crawler"]