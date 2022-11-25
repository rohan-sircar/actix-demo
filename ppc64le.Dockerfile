FROM rust:1.65.0-bullseye

RUN rustup target add powerpc6le-unknown-linux-gnu

RUN apt-get update

RUN apt-get install -y gcc-powerpc64el-linux-gnu

RUN dpkg --add-architecture ppc64el && \
    apt-get update && \
    apt-get install --assume-yes libpq-dev:ppc64el libz-dev:ppc64el