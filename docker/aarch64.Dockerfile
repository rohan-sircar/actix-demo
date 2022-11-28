FROM rust:1.65.0-bullseye

RUN rustup target add aarch64-unknown-linux-gnu

RUN apt-get update

RUN apt-get install -y gcc-aarch64-linux-gnu

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install --assume-yes libpq-dev:arm64 libz-dev:arm64