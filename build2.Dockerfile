FROM debian:bullseye as builder
ARG TARGETPLATFORM
ARG PROFILE=debug

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.65.0

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        ; 

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='5cc9ffd1026e82e7fb2eec2121ad71f4b0f044e88bca39207b3f6b769aaa799c' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='e189948e396d47254103a49c987e7fb0e5dd8e34b200aa4481ecc4b8e41fb929' ;; \
        ppc64el) rustArch='powerpc64le-unknown-linux-gnu'; rustupSha256='774f62fd927f6c29499a6caee8f534e796161321ec35435788971629bb55af8e' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.25.1/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; 

RUN apt-get update && \
    apt-get install -y  \
    libpq-dev \
    libz-dev 

RUN USER=root cargo new --bin actix-demo
WORKDIR /actix-demo

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src
COPY ./build.rs ./build.rs
RUN cargo build

FROM debian:bullseye-slim
ARG APP=/usr/src/app

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