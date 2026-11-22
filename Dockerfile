FROM docker.io/library/rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY openrnet_rs/Cargo.toml ./openrnet_rs/Cargo.toml
COPY src ./src
COPY openrnet_rs/src ./openrnet_rs/src

RUN apt-get update && apt-get install -y --no-install-recommends \
    libdbus-1-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libdbus-1-3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && adduser --disabled-password --gecos "" rnet

COPY --from=builder /app/target/release/rnet-pi-server /usr/local/bin/rnet-pi-server

ENV DBUS_SYSTEM_BUS_ADDRESS=unix:path=/run/dbus/system_bus_socket

USER rnet

ENTRYPOINT ["/usr/local/bin/rnet-pi-server"]
