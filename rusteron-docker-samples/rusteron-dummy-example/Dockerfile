#FROM --platform=linux/amd64 rust:latest AS builder
FROM ubuntu:latest AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y default-jdk curl pkg-config libssl-dev uuid-dev ca-certificates make cmake gcc g++ clang zlib1g-dev libbsd-dev \
    && apt-get clean  \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

COPY Cargo.* ./
COPY src ./src

ENV RUSTFLAGS="-C target-cpu=native"
RUN . "$HOME/.cargo/env" && cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y libssl-dev uuid-dev ca-certificates  \
    && apt-get clean  \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/ticker_reader /usr/local/bin/ticker_reader
COPY --from=builder /usr/src/app/target/release/ticker_writer /usr/local/bin/ticker_writer
COPY --from=builder /usr/src/app/target/release/build/rusteron-archive-*/out/build/lib/*.so /usr/lib

