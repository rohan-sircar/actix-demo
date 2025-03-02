extern crate actix_demo;
use actix_demo::actions::misc::create_database_if_needed;
use actix_demo::models::rate_limit::{
    KeyStrategy, RateLimitConfig, RateLimitPolicy,
};
use actix_demo::models::roles::RoleEnum;
use actix_demo::models::users::{NewUser, Password, Username};
use actix_demo::telemetry::DomainRootSpanBuilder;
use actix_demo::utils::redis_credentials_repo::RedisCredentialsRepo;
use actix_demo::{utils, AppConfig, AppData};
use actix_http::header::HeaderMap;
use actix_web::dev::ServiceResponse;
use actix_web::test::TestRequest;
use actix_web::App;
use actix_web::{test, web};

use actix_web::web::Data;
use anyhow::Context;
use awc::cookie::Cookie;
use awc::{Client, ClientRequest};
use derive_builder::Builder;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use diesel_tracing::pg::InstrumentedPgConnection;
use jwt_simple::prelude::HS256Key;
use once_cell::sync::Lazy;
use std::fs;
use std::io::Write;
use std::os::unix::prelude::OpenOptionsExt;
use testcontainers_modules::postgres::{self, Postgres};
use testcontainers_modules::redis::{Redis, REDIS_PORT};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;
use tracing::subscriber::set_global_default;
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::{format::FmtSpan, Subscriber as FmtSubscriber};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use validators::prelude::*;

use actix_demo::configure_app;

use testcontainers_modules::testcontainers::ImageExt;

use actix_http::{header, Request};
use actix_test::TestServer;
use actix_web::body::MessageBody;
use actix_web::{dev::*, Error as AxError};

pub const DEFAULT_USER: &str = "admin";
pub const X_RATELIMIT_LIMIT: &str = "x-ratelimit-limit";
pub const X_RATELIMIT_REMAINING: &str = "x-ratelimit-remaining";
pub const X_RATELIMIT_RESET: &str = "x-ratelimit-reset";

#[derive(Clone, Debug)]
pub struct BinFile {
    pub location: String,
    pub contents: String,
}

pub fn echo_bin_file() -> BinFile {
    BinFile {
        location: "/tmp/my-echo.sh".to_owned(),
        contents: r#"#!/bin/bash
sleep 2
echo "hello world $1 $2";
"#
        .to_owned(),
    }
}
pub fn sleep_bin_file() -> BinFile {
    BinFile {
        location: "/tmp/sleeper.sh".to_owned(),
        contents: r#"#!/bin/bash
    
    echo "sleeping"
    for i in {1..5}
    do
        echo "$i still sleeping"
        sleep 2
    done
    echo "done sleeping"
    "#
        .to_owned(),
    }
}

pub fn failing_bin_file() -> BinFile {
    BinFile {
        location: "/tmp/failing.sh".to_owned(),
        contents: r#"#!/bin/bash
    
    echo "I'm a failing script"
    exit 1
    "#
        .to_owned(),
    }
}

static TRACING: Lazy<anyhow::Result<()>> = Lazy::new(|| {
    let _ = dotenvy::dotenv().context("Failed to set up env")?;
    let env_filter = EnvFilter::try_from_env("ACTIX_DEMO_TEST_RUST_LOG")
        .context("Failed to set up env logger")?;

    let _ = LogTracer::init()?;

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_test_writer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish()
        .with(env_filter);

    let _ =
        set_global_default(subscriber).context("Failed to set subscriber")?;
    Ok(())
});

static CREATE_BIN_FILES: Lazy<anyhow::Result<()>> = Lazy::new(|| {
    let file1 = echo_bin_file();
    let file2 = sleep_bin_file();
    let file3 = failing_bin_file();
    let files = vec![file1, file2, file3];
    for f in &files {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o777)
            .open(&f.location)?;
        file.write_all(f.contents.as_bytes())?;
        file.flush()?;
    }
    Ok(())
});

#[derive(Clone, Builder, Debug)]
pub struct TestAppOptions {
    #[builder(default = "self.default_bin_file()")]
    pub bin_file: BinFile,
    #[builder(default = "self.default_api_rate_limit()")]
    pub api_rate_limit: RateLimitPolicy,
    #[builder(default = "self.default_auth_rate_limit()")]
    pub auth_rate_limit: RateLimitPolicy,
    #[builder(default = "true")]
    pub rate_limit_disabled: bool,
}

impl Default for TestAppOptions {
    fn default() -> Self {
        TestAppOptionsBuilder::default().build().unwrap()
    }
}

impl TestAppOptionsBuilder {
    fn default_bin_file(&self) -> BinFile {
        echo_bin_file()
    }
    fn default_api_rate_limit(&self) -> RateLimitPolicy {
        RateLimitPolicy {
            max_requests: 1000,
            window_secs: 60,
        }
    }
    fn default_auth_rate_limit(&self) -> RateLimitPolicy {
        RateLimitPolicy {
            max_requests: 1000,
            window_secs: 60,
        }
    }
}

/// Create a new RateLimitConfig with custom settings for tests
pub fn create_rate_limit_config(options: TestAppOptions) -> RateLimitConfig {
    RateLimitConfig {
        key_strategy: KeyStrategy::Random,
        auth: options.auth_rate_limit,
        api: options.api_rate_limit,
        disable: options.rate_limit_disabled,
    }
}

pub async fn app_data(
    pg_connstr: &str,
    redis_connstr: &str,
    options: TestAppOptions,
) -> anyhow::Result<web::Data<AppData>> {
    let _ = Lazy::force(&TRACING).as_ref().unwrap();

    let _ = Lazy::force(&CREATE_BIN_FILES).as_ref().unwrap();

    let config = AppConfig {
        hash_cost: 4,
        job_bin_path: options.bin_file.location.clone(),
        rate_limit: create_rate_limit_config(options),
    };

    let client = redis::Client::open(redis_connstr)
        .context("failed to initialize redis")?;
    let cm = redis::aio::ConnectionManager::new(client.clone())
        .await
        .with_context(|| {
            format!("Failed to connect to redis. Url was: {redis_connstr}",)
        })?;

    let manager =
        ConnectionManager::<InstrumentedPgConnection>::new(pg_connstr);
    let pool = r2d2::Pool::builder()
        .max_size(2)
        .build(manager)
        .context("Failed to create pool")?;

    let _ = {
        let pool = pool.clone();
        let _ = web::block(move || {
            let _ = {
                let mut conn =
                    pool.get().context("Failed to get connection")?;

                let migrations: FileBasedMigrations =
                    FileBasedMigrations::find_migrations_directory()
                        .context("Error running migrations")?;
                let _ = conn
                    .run_pending_migrations(migrations)
                    .map_err(|e| anyhow::anyhow!(e)) // Convert error to anyhow::Error
                    .context("Error running migrations")?;
                actix_demo::actions::users::insert_new_user(
                    NewUser {
                        username: Username::parse_str(DEFAULT_USER)?,
                        password: Password::parse_str(DEFAULT_USER)?,
                    },
                    RoleEnum::RoleAdmin,
                    config.hash_cost,
                    &mut conn,
                )?;
            };

            Ok::<(), anyhow::Error>(())
        })
        .await??;
    };

    let redis_prefix = Box::new(utils::get_redis_prefix("app"));

    let credentials_repo =
        RedisCredentialsRepo::new(redis_prefix(&"user-sessions"), cm.clone());

    let key = HS256Key::from_bytes("test".as_bytes());

    let data = Data::new(AppData {
        config,
        pool,
        credentials_repo,
        jwt_key: key,
        redis_conn_factory: Some(client.clone()),
        redis_conn_manager: Some(cm.clone()),
        redis_prefix,
    });
    Ok(data)
}

pub async fn test_app(
    pg_connstr: &str,
    redis_connstr: &str,
    options: TestAppOptions,
) -> anyhow::Result<
    impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
> {
    let app = App::new()
        .configure(configure_app(
            app_data(pg_connstr, redis_connstr, options).await?,
        ))
        .wrap(TracingLogger::<DomainRootSpanBuilder>::new());
    let test_app = test::init_service(app).await;
    Ok(test_app)
}

pub async fn test_http_app(
    pg_connstr: &str,
    redis_connstr: &str,
    options: TestAppOptions,
) -> anyhow::Result<TestServer> {
    let data = app_data(pg_connstr, redis_connstr, options).await?;
    let test_app = move || {
        App::new()
            .configure(configure_app(data.clone()))
            .wrap(TracingLogger::<DomainRootSpanBuilder>::new())
    };
    Ok(actix_test::start(test_app))
}

pub async fn create_user(
    username: &str,
    password: &str,
    test_app: &impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
) -> String {
    let req = test::TestRequest::post()
        .append_header(("content-type", "application/json"))
        .set_payload(format!(
            r#"{{"username":"{username}","password":"{password}"}}"#
        ))
        .uri("/api/registration")
        .to_request();
    let _ = test_app.call(req).await.unwrap();
    get_token(username, password, test_app).await
}

pub async fn get_token(
    username: &str,
    password: &str,
    test_app: &impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
) -> String {
    let req = test::TestRequest::post()
        .append_header(("content-type", "application/json"))
        .set_payload(format!(
            r#"{{"username":"{username}","password":"{password}"}}"#
        ))
        .uri("/api/login")
        .to_request();
    let resp: ServiceResponse<_> = test_app.call(req).await.unwrap();
    // Get the underlying HttpResponse
    let http_resp = resp.response();

    utils::extract_auth_token(http_resp.headers()).unwrap()
}

pub async fn get_default_token(
    test_app: &impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
) -> String {
    get_token(DEFAULT_USER, DEFAULT_USER, test_app).await
}

pub async fn test_with_postgres(
) -> anyhow::Result<(String, ContainerAsync<Postgres>)> {
    let container = postgres::Postgres::default()
        .with_tag("13-alpine")
        .start()
        .await?;
    let host_port = container.get_host_port_ipv4(5432).await?;
    let connection_string =
        format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",);
    let _ =
        create_database_if_needed(&connection_string).with_context(|| {
            format!(
                "Failed to create/detect database. URL was {connection_string}"
            )
        })?;
    Ok((connection_string, container))
}

pub async fn test_with_redis() -> anyhow::Result<(String, ContainerAsync<Redis>)>
{
    let container = Redis::default().with_tag("7-alpine").start().await?;
    let host = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(REDIS_PORT).await?;
    let connection_string = format!("redis://{host}:{host_port}");
    Ok((connection_string, container))
}

pub trait WithToken {
    fn with_token(self, token: &str) -> Self;
}

impl WithToken for TestRequest {
    fn with_token(self, token: &str) -> Self {
        self.cookie(Cookie::new("X-AUTH-TOKEN", token))
    }
}

impl WithToken for ClientRequest {
    fn with_token(self, token: &str) -> Self {
        self.cookie(Cookie::new("X-AUTH-TOKEN", token))
    }
}

pub async fn get_http_token(
    addr: &str,
    username: &str,
    password: &str,
    client: &Client,
) -> anyhow::Result<String> {
    let resp = client
        .post(format!("http://{addr}/api/login"))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .send_body(format!(
            r#"{{"username":"{username}","password":"{password}"}}"#
        ))
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let token = utils::extract_auth_token(resp.headers())?;
    Ok(token)
}

pub async fn create_http_user(
    addr: &str,
    username: &str,
    password: &str,
    client: &Client,
) -> anyhow::Result<()> {
    let _ = client
        .post(format!("http://{addr}/api/registration"))
        .insert_header(("content-type", "application/json"))
        .send_body(format!(
            r#"{{"username":"{username}","password":"{password}"}}"#
        ))
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    Ok(())
}

pub fn assert_rate_limit_headers(headers: &HeaderMap) {
    // Check for the existence of rate limiting headers
    assert!(
        headers.contains_key(X_RATELIMIT_LIMIT),
        "Expected the '{}' header to be present",
        X_RATELIMIT_LIMIT
    );
    assert!(
        headers.contains_key(X_RATELIMIT_REMAINING),
        "Expected the '{}' header to be present",
        X_RATELIMIT_REMAINING
    );
    assert!(
        headers.contains_key(X_RATELIMIT_RESET),
        "Expected the '{}' header to be present",
        X_RATELIMIT_RESET
    );
}
