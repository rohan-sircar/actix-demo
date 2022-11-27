FROM rust:1.65 as builder

RUN USER=root cargo new --bin actix-demo
WORKDIR /actix-demo

RUN apt-get update && \
    apt-get install -y  \
    libpq-dev \
    libz-dev 

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build 
RUN rm -r src/*.rs

COPY ./src ./src
COPY ./build.rs ./build.rs
RUN rm ./target/debug/deps/actix_demo*
RUN cargo build 

FROM debian:bullseye-slim
ARG APP=/usr/src/app
ARG TARGETOS
ARG TARGETARCH

RUN apt-get update && \
    apt-get install -y ca-certificates \
    tzdata libpq-dev \
    libz-dev && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 7800

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY ./.env ${APP}/.env
COPY ./migrations ${APP}/migrations
COPY ./static ${APP}/static
COPY --from=builder /actix-demo/target/debug/actix-demo ${APP}/actix-demo

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}
CMD ["./actix-demo"]