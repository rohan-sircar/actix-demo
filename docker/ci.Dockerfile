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
COPY ./target/${TARGETARCH}-${TARGETOS}/actix-demo ${APP}/actix-demo

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}
CMD ["./actix-demo"]