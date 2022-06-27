FROM --platform=$BUILDPLATFORM dockcross/base AS builder
ARG TARGETARCH

WORKDIR /app

COPY docker/platform.sh .

RUN ./platform.sh

COPY . .

RUN apt-get update

RUN apt-get install -y \
    build-essential \
    curl

RUN apt-get update

ENV GRPC_HOST=0.0.0.0:50055
ENV GRPC_HOST_API=api:50051

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_ALL_STATIC=1

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup component add rustfmt
RUN rustup target add $(cat /.platform) 
RUN apt-get update && apt-get install -y --no-install-recommends build-essential libssl-dev unzip $(cat /.compiler)
RUN apt-get update

RUN cargo install  --path . --no-default-features

FROM --platform=$BUILDPLATFORM debian:bullseye-slim

ENV GRPC_HOST=0.0.0.0:50055
ENV GRPC_HOST_API=api:50051

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential \
    ca-certificates libssl-dev

COPY --from=builder /root/.cargo/bin/website_crawler /usr/local/bin/website_crawler
COPY --from=builder /root/.cargo/bin/health_client /usr/local/bin/health_client

CMD ["website_crawler"]