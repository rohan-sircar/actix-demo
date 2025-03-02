FROM --platform=${BUILDPLATFORM} rust:1.85 as builder
ARG TARGETPLATFORM
ARG PROFILE=debug

RUN \ 
    case ${TARGETPLATFORM} in \
    "linux/amd64") TARGET="x86_64-unknown-linux-gnu"  ;; \
    "linux/arm64") TARGET="aarch64-unknown-linux-gnu"  ;; \
    "linux/ppc64le") TARGET="powerpc64le-unknown-linux-gnu"  ;; \
    esac && \
    rustup target add $TARGET

RUN \ 
    case ${TARGETPLATFORM} in \
    "linux/amd64") GCC="gcc"  ;; \
    "linux/arm64") GCC="gcc-aarch64-linux-gnu"  ;; \
    "linux/ppc64le") GCC="gcc-powerpc64le-linux-gnu"  ;; \
    esac && \
    apt-get update && \
    apt-get install -y ${GCC}

RUN \ 
    case ${TARGETPLATFORM} in \
    "linux/amd64") ARCH="amd64"  ;; \
    "linux/arm64") ARCH="arm64"  ;; \
    "linux/ppc64le") ARCH="ppc64el"  ;; \
    esac && \
    dpkg --add-architecture ${ARCH} &&\
    apt-get update && \
    apt-get install -y  \
    libpq-dev:${ARCH} \
    libz-dev:${ARCH} 


RUN USER=root cargo new --bin actix-demo
WORKDIR /actix-demo

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN \
    case ${PROFILE} in \
    "debug") CARGOFLAGS=""  ;; \
    "release") CARGOFLAGS="--release"  ;; \
    esac && \ 
    case ${TARGETPLATFORM} in \
    "linux/amd64") BUILDFLAGS=""  ;; \
    "linux/arm64") BUILDFLAGS="-C linker=aarch64-linux-gnu-gcc"  ;; \
    "linux/ppc64le") BUILDFLAGS="-C linker=powerpc64le-linux-gnu-gcc"  ;; \
    esac && \
    case ${TARGETPLATFORM} in \
    "linux/amd64") TARGET="x86_64-unknown-linux-gnu"  ;; \
    "linux/arm64") TARGET="aarch64-unknown-linux-gnu"  ;; \
    "linux/ppc64le") TARGET="powerpc64le-unknown-linux-gnu"  ;; \
    esac && \
    RUSTFLAGS="${BUILDFLAGS}" cargo build --target ${TARGET} $CARGOFLAGS
RUN rm -r src/*.rs
COPY ./src ./src
COPY ./build.rs ./build.rs
RUN \
    case ${PROFILE} in \
    "debug") CARGOFLAGS=""  ;; \
    "release") CARGOFLAGS="--release"  ;; \
    esac && \ 
    case ${TARGETPLATFORM} in \
    "linux/amd64") BUILDFLAGS=""  ;; \
    "linux/arm64") BUILDFLAGS="-C linker=aarch64-linux-gnu-gcc"  ;; \
    "linux/ppc64le") BUILDFLAGS="-C linker=powerpc64le-linux-gnu-gcc"  ;; \
    esac && \
    case ${TARGETPLATFORM} in \
    "linux/amd64") TARGET="x86_64-unknown-linux-gnu"  ;; \
    "linux/arm64") TARGET="aarch64-unknown-linux-gnu"  ;; \
    "linux/ppc64le") TARGET="powerpc64le-unknown-linux-gnu"  ;; \
    esac && \
    RUSTFLAGS="${BUILDFLAGS}" cargo build --target ${TARGET} $CARGOFLAGS
RUN \
    case ${PROFILE} in \
    "debug") RELEASEPATH="debug"  ;; \
    "release") RELEASEPATH="release"  ;; \
    esac && \
    case ${TARGETPLATFORM} in \
    "linux/amd64")  cp target/x86_64-unknown-linux-gnu/${RELEASEPATH}/actix-demo target/actix-demo ;;\
    "linux/arm64") cp target/aarch64-unknown-linux-gnu/${RELEASEPATH}/actix-demo target/actix-demo ;;\
    "linux/ppc64le") cp target/powerpc64le-unknown-linux-gnu/${RELEASEPATH}/actix-demo target/actix-demo ;; \
    esac 

FROM debian:stable-slim
ARG APP=/usr/src/app

RUN apt-get update && \
    apt-get install -y ca-certificates \
    tzdata libpq-dev \
    libz-dev 

EXPOSE 7800

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY ./.env ${APP}/.env
COPY ./migrations ${APP}/migrations
COPY ./static ${APP}/static
COPY --from=builder /actix-demo/target/actix-demo ${APP}/actix-demo

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}
CMD ["./actix-demo"]