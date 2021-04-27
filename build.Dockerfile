FROM rust:1.51 as builder

# ENV CARGO_HOME=/actix-demo/.cargo
RUN USER=root cargo new --bin actix-demo
WORKDIR /actix-demo

# COPY ./.cargo ./.cargo
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release
RUN rm -r src/*.rs

COPY ./src ./src
RUN rm ./target/release/deps/actix_demo*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY ./.env ${APP}/.env
COPY ./migrations ${APP}/migrations
COPY ./static ${APP}/static
COPY ./db/empty.db ${APP}/data/app.db
COPY --from=builder /actix-demo/target/release/actix-demo ${APP}/actix-demo

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}
CMD ["./actix-demo"]