FROM rust:1.67.0-alpine3.16 AS builder

WORKDIR /app
COPY ../ .

ENV GRPC_HOST=0.0.0.0:50055
ENV GRPC_HOST_API=api:50051

RUN apk add --update \
    build-base cmake make libc6-compat make protoc protobuf-dev pkgconfig openssl openssl-dev libffi-dev zlib-dev musl-dev && \
    rm -rf /var/cache/apk/*

RUN cargo install --path .

FROM alpine:3.16

RUN apk upgrade  \
    && apk add \
    libc6-compat openssl && \
    rm -rf /var/cache/apk/*

COPY --from=builder /usr/local/cargo/bin/website_crawler /usr/local/bin/website_crawler
COPY --from=builder /usr/local/cargo/bin/health_client /usr/local/bin/health_client

ENV GRPC_HOST=0.0.0.0:50055
ENV GRPC_HOST_API=api:50051

CMD ["website_crawler"]