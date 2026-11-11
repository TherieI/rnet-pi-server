FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY openrnet_rs/Cargo.toml ./openrnet_rs/Cargo.toml
COPY src ./src
COPY openrnet_rs/src ./openrnet_rs/src

RUN cargo build --release --locked

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/rnet-pi-server /usr/local/bin/rnet-pi-server

ENTRYPOINT ["/usr/local/bin/rnet-pi-server"]
