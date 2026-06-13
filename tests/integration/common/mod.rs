extern crate actix_demo;
use actix_demo::actions::misc::create_database_if_needed;
use actix_demo::config::{MinioConfig, OAuthConfig, TlsMode};
use actix_demo::models::rate_limit::{
    KeyStrategy, RateLimitConfig, RateLimitPolicy,
};
use actix_demo::models::roles::RoleEnum;
use actix_demo::models::session::{
    SessionConfig, SessionConfigBuilder, SessionInfo,
};
use actix_demo::models::users::{Email, NewUser, Password, User, Username};
use actix_demo::models::worker::{WorkerBackoffConfig, WorkerConfig};
use actix_demo::services::email::smtp::SmtpSender;
use actix_demo::telemetry::DomainRootSpanBuilder;
pub use actix_demo::utils;
use actix_demo::utils::redis_credentials_repo::RedisCredentialsRepo;
use actix_demo::utils::InstrumentedRedisCache;
use actix_demo::{AppConfig, AppData, SmtpConfig};
use actix_http::header::HeaderMap;
use actix_web::dev::ServiceResponse;
use actix_web::test::TestRequest;
use actix_web::App;
use actix_web::{test, web};

use actix_web::web::Data;
use actix_web_prom::PrometheusMetricsBuilder;
use anyhow::Context;
use awc::cookie::Cookie;
use awc::{Client, ClientRequest};
use cached::RedisCacheBuilder;
use derive_builder::Builder;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use diesel_tracing::pg::InstrumentedPgConnection;
use jwt_simple::prelude::HS256Key;
use minior::aws_sdk_s3;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::unix::prelude::OpenOptionsExt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use testcontainers_modules::minio::{self, MinIO};
use testcontainers_modules::postgres::{self, Postgres};
use testcontainers_modules::redis::{Redis, REDIS_PORT};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::Image;
use tracing::subscriber::set_global_default;
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::{format::FmtSpan, Subscriber as FmtSubscriber};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use validators::prelude::*;

use actix_demo::configure_app;

use testcontainers_modules::testcontainers::ImageExt;

/// Mailpit test container for email testing
pub struct Mailpit {
    tag: &'static str,
}

impl Default for Mailpit {
    fn default() -> Self {
        Mailpit { tag: "latest" }
    }
}

impl Image for Mailpit {
    fn name(&self) -> &str {
        "axllent/mailpit"
    }

    fn tag(&self) -> &str {
        self.tag
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![testcontainers::core::WaitFor::healthcheck()]
    }
}

/// Start a Mailpit container and return (smtp_host, http_port, container)
pub async fn test_with_mailpit(
) -> anyhow::Result<(String, u16, ContainerAsync<Mailpit>)> {
    let container = Mailpit::default().start().await?;
    let smtp_port = container.get_host_port_ipv4(1025).await?;
    let http_port = container.get_host_port_ipv4(8025).await?;

    // Wait for SMTP port to be ready (healthcheck only verifies HTTP)
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(100);
    loop {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{smtp_port}"))
            .await
            .is_ok()
        {
            break;
        }
        if start.elapsed() >= Duration::from_secs(10) {
            anyhow::bail!("SMTP port {} not ready", smtp_port);
        }
        tokio::time::sleep(poll_interval).await;
    }

    let smtp_host = format!("127.0.0.1:{smtp_port}");
    Ok((smtp_host, http_port, container))
}

/// Mailpit HTTP API client for interacting with the test mail server
pub struct MailpitClient {
    pub base_url: String,
    pub http: reqwest::Client,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct MessageSummary {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "From")]
    pub from: Address,
    #[serde(rename = "To")]
    pub to: Vec<Address>,
    #[serde(rename = "Subject")]
    pub subject: String,
    #[serde(rename = "Text")]
    pub text: Option<String>,
    #[serde(rename = "HTML")]
    pub html: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Address {
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "Name")]
    pub display_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MessagesResponse {
    pub messages: Vec<MessageSummary>,
}

/// Full message details (from /api/v1/message/{ID} endpoint)
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Message {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "From")]
    pub from: Address,
    #[serde(rename = "To")]
    pub to: Vec<Address>,
    #[serde(rename = "Subject")]
    pub subject: String,
    #[serde(rename = "Text")]
    pub text: Option<String>,
    #[serde(rename = "HTML")]
    pub html: Option<String>,
}

impl MailpitClient {
    pub fn new(http_port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{http_port}"),
            http: reqwest::Client::new(),
        }
    }

    /// Delete all messages in the mailbox (cleanup between tests)
    pub async fn delete_all(&self) -> anyhow::Result<()> {
        self.http
            .delete(format!("{}/api/v1/messages", self.base_url))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Wait for a verification email to arrive and extract the token from it
    pub async fn wait_for_verification_token(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<String> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(200);
        let token_regex = Regex::new(r"token=([^&\s]+)").unwrap();
        loop {
            let resp = self
                .http
                .get(format!("{}/api/v1/messages?limit=1", self.base_url))
                .send()
                .await?
                .json::<MessagesResponse>()
                .await?;

            if let Some(msg_summary) = resp.messages.into_iter().next() {
                if msg_summary.subject.contains("Verify") {
                    // Fetch full message details to get the body
                    let msg = self
                        .http
                        .get(format!(
                            "{}/api/v1/message/{}",
                            self.base_url, msg_summary.id
                        ))
                        .send()
                        .await?
                        .json::<Message>()
                        .await?;
                    let text = msg
                        .text
                        .context("verification email has no text body")?;
                    return token_regex
                        .captures(&text)
                        .and_then(|c| c.get(1))
                        .map(|m| m.as_str().to_string())
                        .ok_or_else(|| {
                            anyhow::anyhow!("could not find token in verification email text")
                        });
                }
            }

            if start.elapsed() >= timeout {
                anyhow::bail!("timed out waiting for verification email");
            }
            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Wait for any email to arrive and return the full message details
    pub async fn wait_for_email(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<Message> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(200);
        loop {
            let resp = self
                .http
                .get(format!("{}/api/v1/messages?limit=1", self.base_url))
                .send()
                .await?
                .json::<MessagesResponse>()
                .await?;

            if let Some(msg_summary) = resp.messages.into_iter().next() {
                // Fetch full message details
                let msg = self
                    .http
                    .get(format!(
                        "{}/api/v1/message/{}",
                        self.base_url, msg_summary.id
                    ))
                    .send()
                    .await?
                    .json::<Message>()
                    .await?;
                return Ok(msg);
            }

            if start.elapsed() >= timeout {
                anyhow::bail!("timed out waiting for email");
            }
            tokio::time::sleep(poll_interval).await;
        }
    }
}

use actix_http::{header, Request, StatusCode};
use actix_test::TestServer;
use actix_web::body::MessageBody;
use actix_web::{dev::*, Error as AxError};

pub const DEFAULT_USER: &str = "admin";
pub const X_RATELIMIT_LIMIT: &str = "x-ratelimit-limit";
pub const X_RATELIMIT_REMAINING: &str = "x-ratelimit-remaining";
pub const X_RATELIMIT_RESET: &str = "x-ratelimit-reset";
pub const JWT_SECRET_KEY: &[u8] = b"test-jwt-secret-key-at-least-12-bytes";

lazy_static::lazy_static! {
    pub static ref TEST_JWT_KEY: HS256Key = HS256Key::from_bytes(JWT_SECRET_KEY);
}

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
        // .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_span_events(FmtSpan::NONE)
        .with_env_filter(env_filter)
        .finish();

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
    #[builder(default = "self.default_session_config()")]
    pub session_config: SessionConfig,
    #[builder(default = "self.default_session_cleanup_worker_config()")]
    pub sessions_cleanup_worker_config: WorkerConfig,
    #[builder(default = "None")]
    pub smtp_host: Option<String>,
    #[builder(default = "None")]
    pub oauth_config: Option<OAuthConfig>,
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
    fn default_session_config(&self) -> SessionConfig {
        SessionConfigBuilder::default().build().unwrap()
    }
    fn default_session_cleanup_worker_config(&self) -> WorkerConfig {
        WorkerConfig {
            backoff: WorkerBackoffConfig {
                initial_interval_secs: 1,
                multiplier: 1.0,
                max_interval_secs: 30,
                max_elapsed_time_secs: 30,
            },
            run_interval: 2,
        }
    }
}

/// Create a new RateLimitConfig with custom settings for tests
pub fn create_rate_limit_config(options: TestAppOptions) -> RateLimitConfig {
    RateLimitConfig {
        key_strategy: KeyStrategy::Random,
        auth: options.auth_rate_limit,
        api: options.api_rate_limit,
        api_public: RateLimitPolicy {
            max_requests: 15,
            window_secs: 60,
        },
        disable: options.rate_limit_disabled,
    }
}

pub async fn app_data(
    pg_connstr: &str,
    redis_connstr: &str,
    minio_connstr: &str,
    options: TestAppOptions,
) -> anyhow::Result<web::Data<AppData>> {
    let start_time = SystemTime::now();
    let _ = Lazy::force(&TRACING).as_ref().unwrap();

    let _ = Lazy::force(&CREATE_BIN_FILES).as_ref().unwrap();

    let smtp_host = options.smtp_host.clone();

    let config = AppConfig {
        hash_cost: 4,
        job_bin_path: options.bin_file.location.clone(),
        rate_limit: create_rate_limit_config(options.clone()),
        session: options.session_config.clone(),
        health_check_timeout_secs: 10,
        minio: MinioConfig {
            bucket_name: "actix-demo".to_owned(),
            max_avatar_size_bytes:
                actix_demo::config::default_avatar_size_limit(),
            max_pet_image_size_bytes:
                actix_demo::config::default_pet_image_size_limit(),
        },
        timezone: chrono_tz::Tz::UTC,
        email_token_ttl_verification_secs: 86400,
        email_token_ttl_reset_secs: 900,
        smtp: {
            let smtp = smtp_host
                .clone()
                .unwrap_or_else(|| "localhost:1025".to_string());
            let (host, port) = smtp
                .rsplit_once(':')
                .map(|(h, p)| (h.to_string(), p.parse().unwrap_or(1025u16)))
                .unwrap_or_else(|| ("localhost".to_string(), 1025u16));
            SmtpConfig {
                host,
                port,
                username: "".to_string(),
                password: "".to_string(),
                from_email: "noreply@example.com".to_string(),
                tls_mode: TlsMode::None,
            }
        },
        oauth: options.oauth_config.unwrap_or(OAuthConfig {
            enabled: false,
            base_url: "http://localhost:7800".to_string(),
            github_client_id: String::new(),
            github_client_secret: String::new(),
            github_scopes: Vec::new(),
            google_client_id: String::new(),
            google_client_secret: String::new(),
            google_scopes: Vec::new(),
        }),
        cors_origins: "*".to_string(),
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

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();
    let metrics =
        actix_demo::metrics::Metrics::new(prometheus.clone().registry);

    let user_ids_cache = InstrumentedRedisCache::new(
        RedisCacheBuilder::new("test_user_ids", Duration::from_secs(3600))
            .connection_string(redis_connstr)
            .build()
            .unwrap(),
        metrics.cache.clone(),
    );

    let _ = {
        let pool = pool.clone();
        let user_ids_cache = user_ids_cache.clone();
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
                        email: Email::try_from("admin@example.com".to_string())
                            .unwrap(),
                    },
                    RoleEnum::RoleAdmin,
                    config.hash_cost,
                    &user_ids_cache,
                    &mut conn,
                )?;
            };

            Ok::<(), anyhow::Error>(())
        })
        .await??;
    };

    let redis_prefix = Box::new(utils::get_redis_prefix("app"));

    let credentials_repo = RedisCredentialsRepo::new(
        redis_prefix(&"user-sessions"),
        cm.clone(),
        options.session_config.max_concurrent_sessions,
        options.session_config.renewal.renewal_window_secs,
        metrics.active_sessions.clone(),
    );

    let key = TEST_JWT_KEY.clone();

    // Create MinIO client
    let cred = aws_sdk_s3::config::Credentials::new(
        "minioadmin",
        "minioadmin",
        None,
        None,
        "testcontainers",
    );
    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(minio_connstr)
        .credentials_provider(cred)
        .region(aws_sdk_s3::config::Region::new("test"))
        .force_path_style(true) // apply bucketname as path param instead of pre-domain
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .build();
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config.clone());

    // Create MinIO bucket if it doesn't exist
    let bucket_name = "actix-demo";
    let _ = s3_client.create_bucket().bucket(bucket_name).send().await;

    let data = Data::new(AppData {
        start_time,
        config,
        pool,
        credentials_repo,
        jwt_key: key,
        redis_conn_factory: client.clone(),
        redis_conn_manager: cm.clone(),
        redis_prefix,
        sessions_cleanup_worker_handle: None,
        metrics,
        prometheus,
        user_ids_cache,
        health_checkers: Vec::new(),
        minio: minior::Minio {
            client: Arc::new(s3_client),
        },
        mailer: {
            let smtp = options
                .smtp_host
                .clone()
                .unwrap_or_else(|| "localhost:587".to_string());
            let (host, port) = smtp
                .rsplit_once(':')
                .map(|(h, p)| (h.to_string(), p.parse().unwrap_or(587u16)))
                .unwrap_or_else(|| ("localhost".to_string(), 587u16));
            Arc::new(
                SmtpSender::new(
                    &actix_demo::services::email::smtp::SmtpSenderConfig {
                        smtp_host: host,
                        smtp_port: port,
                        tls_mode: TlsMode::None,
                        username: "".to_string(),
                        password: "".to_string(),
                        from_email: "noreply@example.com".to_string(),
                        verification_link_template: "https://app.example.com/verify?token={token}&user={user_name}".to_string(),
                        password_reset_link_template: "https://app.example.com/reset?token={token}&user={user_name}".to_string(),
      },
                    )
                .unwrap(),
            )
        },
        swagger_path: "/api/v1/swagger".to_string(),
    });
    Ok(data)
}

#[allow(dead_code)]
pub async fn test_app(
    pg_connstr: &str,
    redis_connstr: &str,
    minio_connstr: &str,
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
            app_data(pg_connstr, redis_connstr, minio_connstr, options).await?,
        ))
        .wrap(TracingLogger::<DomainRootSpanBuilder>::new());
    let test_app = test::init_service(app).await;
    Ok(test_app)
}

pub async fn test_http_app(
    pg_connstr: &str,
    redis_connstr: &str,
    minio_connstr: &str,
    options: TestAppOptions,
) -> anyhow::Result<(TestServer, web::Data<AppData>)> {
    let data =
        app_data(pg_connstr, redis_connstr, minio_connstr, options).await?;
    let data_clone = data.clone();
    let test_app = move || {
        App::new()
            .configure(configure_app(data_clone.clone()))
            .wrap(TracingLogger::<DomainRootSpanBuilder>::new())
    };
    Ok((actix_test::start(test_app), data))
}

#[allow(dead_code)]
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
        .uri("/api/v1/login")
        .to_request();
    let resp: ServiceResponse<_> = test_app.call(req).await.unwrap();
    // Get the underlying HttpResponse
    let http_resp = resp.response();

    utils::extract_auth_token(http_resp.headers()).unwrap()
}

#[allow(dead_code)]
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

pub async fn test_with_minio() -> anyhow::Result<(String, ContainerAsync<MinIO>)>
{
    let container = minio::MinIO::default().start().await?;
    let host_port = container.get_host_port_ipv4(9000).await?;
    let connection_string = format!("http://127.0.0.1:{host_port}");
    Ok((connection_string, container))
}

pub trait WithToken {
    fn with_token(self, token: &str) -> Self;
    fn with_bearer(self, token: &str) -> Self;
}

impl WithToken for TestRequest {
    fn with_token(self, token: &str) -> Self {
        self.cookie(Cookie::new("X-AUTH-TOKEN", token))
    }

    fn with_bearer(self, token: &str) -> Self {
        self.insert_header(("Authorization", format!("Bearer {token}")))
    }
}

impl WithToken for ClientRequest {
    fn with_token(self, token: &str) -> Self {
        self.cookie(Cookie::new("X-AUTH-TOKEN", token))
    }

    fn with_bearer(self, token: &str) -> Self {
        self.insert_header(("Authorization", format!("Bearer {token}")))
    }
}

pub async fn get_http_token(
    addr: &str,
    username: &str,
    password: &str,
    client: &Client,
) -> anyhow::Result<String> {
    let resp = client
        .post(format!("http://{addr}/api/v1/login"))
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
    let email = format!("{}@test.local", username);
    create_http_user_with_email(addr, username, password, &email, client).await
}

pub async fn create_http_user_with_email(
    addr: &str,
    username: &str,
    password: &str,
    email: &str,
    client: &Client,
) -> anyhow::Result<()> {
    let mut resp = client
        .post(format!("http://{addr}/api/v1/registration"))
        .insert_header(("content-type", "application/json"))
        .send_json(&serde_json::json!({
            "username": username,
            "password": password,
            "email": email
        }))
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    let status = resp.status();
    if status != actix_web::http::StatusCode::CREATED {
        let body = resp
            .json::<serde_json::Value>()
            .await
            .unwrap_or(serde_json::json!({"raw": "unreadable"}));
        eprintln!("REGISTRATION ERROR status={} body={}", status, body);
        anyhow::bail!("Registration failed with status {}: {}", status, body);
    }

    Ok(())
}

pub async fn register_and_login(
    ctx: &TestContext,
    username: &str,
    password: &str,
) -> String {
    create_http_user(&ctx.addr, username, password, &ctx.client)
        .await
        .unwrap();

    get_http_token(&ctx.addr, username, password, &ctx.client)
        .await
        .unwrap()
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
pub struct TestContext {
    pub addr: String,
    pub _token: String,
    pub client: Client,
    pub _pg: ContainerAsync<Postgres>,
    pub _redis: ContainerAsync<Redis>,
    pub _minio: ContainerAsync<MinIO>,
    pub test_server: TestServer,
    pub app_data: web::Data<AppData>,
    pub _mailpit: Option<ContainerAsync<Mailpit>>,
    pub mailpit_client: Option<MailpitClient>,
}

impl TestContext {
    pub async fn new(options: Option<TestAppOptions>) -> Self {
        let (pg_connstr, _pg) = test_with_postgres().await.unwrap();
        let (redis_connstr, _redis) = test_with_redis().await.unwrap();
        let (minio_connstr, _minio) = test_with_minio().await.unwrap();

        let (test_server, app_data) = test_http_app(
            &pg_connstr,
            &redis_connstr,
            &minio_connstr,
            options.unwrap_or_default(),
        )
        .await
        .unwrap();

        let addr = test_server.addr().to_string();
        let client = Client::new();

        let _token = get_http_token(&addr, DEFAULT_USER, DEFAULT_USER, &client)
            .await
            .unwrap();

        Self {
            addr,
            _token,
            client,
            _pg,
            _redis,
            _minio,
            test_server,
            app_data,
            _mailpit: None,
            mailpit_client: None,
        }
    }

    pub async fn new_with_mailpit(options: Option<TestAppOptions>) -> Self {
        let (smtp_host, http_port, _mailpit) =
            test_with_mailpit().await.unwrap();
        let (pg_connstr, _pg) = test_with_postgres().await.unwrap();
        let (redis_connstr, _redis) = test_with_redis().await.unwrap();
        let (minio_connstr, _minio) = test_with_minio().await.unwrap();

        let options = {
            let mut builder = TestAppOptionsBuilder::default();
            if let Some(opts) = options {
                builder
                    .bin_file(opts.bin_file)
                    .api_rate_limit(opts.api_rate_limit)
                    .auth_rate_limit(opts.auth_rate_limit)
                    .rate_limit_disabled(opts.rate_limit_disabled)
                    .session_config(opts.session_config)
                    .sessions_cleanup_worker_config(
                        opts.sessions_cleanup_worker_config,
                    );
            }
            builder.smtp_host(Some(smtp_host)).build().unwrap()
        };

        let (test_server, app_data) =
            test_http_app(&pg_connstr, &redis_connstr, &minio_connstr, options)
                .await
                .unwrap();

        let addr = test_server.addr().to_string();
        let client = Client::new();

        let _token = get_http_token(&addr, DEFAULT_USER, DEFAULT_USER, &client)
            .await
            .unwrap();

        Self {
            addr,
            _token,
            client,
            _pg,
            _redis,
            _minio,
            test_server,
            app_data,
            _mailpit: Some(_mailpit),
            mailpit_client: Some(MailpitClient::new(http_port)),
        }
    }

    pub async fn create_tokens(&self, count: usize) -> Vec<String> {
        let mut tokens = Vec::new();

        for _ in 0..count {
            let token = get_http_token(
                &self.addr,
                DEFAULT_USER,
                DEFAULT_USER,
                &self.client,
            )
            .await
            .unwrap();
            tokens.push(token);
        }

        tokens
    }

    pub async fn get_sessions(
        &self,
        token: &str,
    ) -> HashMap<Uuid, SessionInfo> {
        let mut resp = self
            .test_server
            .get("/api/v1/sessions")
            .with_token(token)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "Failed to get sessions");
        resp.json().await.unwrap()
    }

    pub async fn delete_session(&self, session_id: Uuid, token: &str) {
        let resp = self
            .test_server
            .delete(format!("/api/v1/sessions/{}", session_id))
            .with_token(token)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "Failed to delete session");
    }

    pub async fn _get_users(
        &self,
        page: i8,
        limit: i8,
        token: &str,
    ) -> Vec<User> {
        let mut resp = self
            .test_server
            .get(format!("/api/v1/admin/users?page={page}&limit={limit}"))
            .with_token(token)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "Failed to get users");
        resp.json().await.unwrap()
    }
}

pub fn assert_session_headers(headers: &HeaderMap) {
    assert!(
        headers.contains_key("x-session-id"),
        "Missing session ID header"
    );
    assert!(
        headers.contains_key("x-session-device-id"),
        "Missing device ID header"
    );
    assert!(
        headers.contains_key("x-session-created-at"),
        "Missing created at header"
    );
    assert!(
        headers.contains_key("x-session-last-used-at"),
        "Missing last used header"
    );
    assert!(
        headers.contains_key("x-session-ttl-remaining"),
        "Missing TTL remaining header"
    );
}

pub fn get_ttl_remaining(headers: &HeaderMap) -> Option<i64> {
    headers
        .get("x-session-ttl-remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
}

pub fn get_session_metadata(headers: &HeaderMap) -> Option<(String, String)> {
    let session_id =
        headers.get("x-session-id").and_then(|v| v.to_str().ok())?;
    let device_id = headers
        .get("x-session-device-id")
        .and_then(|v| v.to_str().ok())?;
    Some((session_id.to_string(), device_id.to_string()))
}

pub fn get_last_used_timestamp(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-session-last-used-at")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
