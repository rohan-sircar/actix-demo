use actix_demo::config::TlsMode;
use actix_demo::config::{OAuthConfig, OAuthProviderConfig};
use actix_demo::configure_app;
use actix_demo::models::roles::RoleEnum;
use actix_demo::models::session::SessionConfigBuilder;
use actix_demo::models::users::{Email, NewUser, Password, Username};
use validators::traits::ValidateString;
use actix_demo::services::email::smtp::SmtpSender;
use actix_demo::telemetry::DomainRootSpanBuilder;
use actix_demo::utils::redis_credentials_repo::RedisCredentialsRepo;
use actix_demo::utils::InstrumentedRedisCache;
use actix_demo::{AppConfig, AppData};
use actix_test::TestServer;
use actix_web::web::Data;
use actix_web::App;
use awc::Client;
use cached::RedisCacheBuilder;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use diesel_tracing::pg::InstrumentedPgConnection;
use minior::aws_sdk_s3;
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::minio::MinIO;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::ImageExt;
use tracing_actix_web::TracingLogger;
use wiremock::MockServer;

use crate::common::{self, TestAppOptionsBuilder, DEFAULT_USER};
use crate::common::{test_with_minio, test_with_postgres, test_with_redis};

use actix_demo::actions::misc::create_database_if_needed;
use actix_demo::actions::users::insert_new_user;

pub struct OAuthTestContext {
    pub addr: String,
    pub client: Client,
    pub _pg: ContainerAsync<Postgres>,
    pub _redis: ContainerAsync<Redis>,
    pub _minio: ContainerAsync<MinIO>,
    pub test_server: TestServer,
    pub app_data: Data<AppData>,
}

pub async fn setup_oauth_app(mock_server: &MockServer) -> OAuthTestContext {
    setup_oauth_app_with_servers(mock_server, mock_server).await
}

pub async fn setup_oauth_app_with_servers(
    github_server: &MockServer,
    _google_server: &MockServer,
) -> OAuthTestContext {
    let (pg_connstr, _pg) = test_with_postgres().await.unwrap();
    let (redis_connstr, _redis) = test_with_redis().await.unwrap();
    let (minio_connstr, _minio) = test_with_minio().await.unwrap();

    let oauth_config = OAuthConfig {
        enabled: true,
        base_url: format!("http://{}", github_server.address()),
        github: OAuthProviderConfig {
            client_id: "test-github-client-id".to_string(),
            client_secret: "test-github-client-secret".to_string(),
            scopes: vec!["user:email".to_string()],
        },
        google: OAuthProviderConfig {
            client_id: "test-google-client-id".to_string(),
            client_secret: "test-google-client-secret".to_string(),
            scopes: vec!["openid email profile".to_string()],
        },
    };

    let data = create_app_data(
        &pg_connstr,
        &redis_connstr,
        &minio_connstr,
        oauth_config,
    )
    .await;

    let test_server = {
        let data_clone = data.clone();
        actix_test::start(move || {
            App::new()
                .configure(configure_app(data_clone.clone()))
                .wrap(TracingLogger::<DomainRootSpanBuilder>::new())
        })
    };

    let addr = test_server.addr().to_string();
    let client = Client::builder().disable_redirects().finish();

    OAuthTestContext {
        addr,
        client,
        _pg,
        _redis,
        _minio,
        test_server,
        app_data: data,
    }
}

pub async fn setup_disabled_oauth_app(
    pg_connstr: &str,
    redis_connstr: &str,
    minio_connstr: &str,
) -> OAuthTestContext {
    let _pg = Postgres::default()
        .with_tag("13-alpine")
        .start()
        .await
        .unwrap();
    let _redis = Redis::default().with_tag("7-alpine").start().await.unwrap();
    let _minio = MinIO::default().start().await.unwrap();

    let oauth_config = OAuthConfig {
        enabled: false,
        base_url: "http://localhost:0".to_string(),
        github: OAuthProviderConfig::default(),
        google: OAuthProviderConfig::default(),
    };

    let data =
        create_app_data(pg_connstr, redis_connstr, minio_connstr, oauth_config)
            .await;

    let test_server = {
        let data_clone = data.clone();
        actix_test::start(move || {
            App::new()
                .configure(configure_app(data_clone.clone()))
                .wrap(TracingLogger::<DomainRootSpanBuilder>::new())
        })
    };

    let addr = test_server.addr().to_string();
    let client = Client::builder().disable_redirects().finish();

    OAuthTestContext {
        addr,
        client,
        _pg,
        _redis,
        _minio,
        test_server,
        app_data: data,
    }
}

async fn create_app_data(
    pg_connstr: &str,
    redis_connstr: &str,
    minio_connstr: &str,
    oauth_config: OAuthConfig,
) -> Data<AppData> {
    let config = AppConfig {
        hash_cost: 4,
        job_bin_path: "/tmp/test.bin".to_string(),
        rate_limit: common::create_rate_limit_config(
            TestAppOptionsBuilder::default().build().unwrap(),
        ),
        session: SessionConfigBuilder::default().build().unwrap(),
        health_check_timeout_secs: 10,
        minio: actix_demo::config::MinioConfig {
            bucket_name: "actix-demo".to_string(),
            max_avatar_size_bytes:
                actix_demo::config::default_avatar_size_limit(),
        },
        timezone: chrono_tz::Tz::UTC,
        email_token_ttl_verification_secs: 86400,
        email_token_ttl_reset_secs: 900,
        smtp: actix_demo::SmtpConfig {
            host: "localhost".to_string(),
            port: 1025,
            username: "".to_string(),
            password: "".to_string(),
            from_email: "noreply@example.com".to_string(),
            tls_mode: TlsMode::None,
        },
        oauth: oauth_config,
    };

    do_create_app_data(pg_connstr, redis_connstr, minio_connstr, config).await
}

async fn do_create_app_data(
    pg_connstr: &str,
    redis_connstr: &str,
    minio_connstr: &str,
    config: AppConfig,
) -> Data<AppData> {
    let _ = create_database_if_needed(pg_connstr).unwrap();

    let client = redis::Client::open(redis_connstr).unwrap();
    let cm = redis::aio::ConnectionManager::new(client.clone())
        .await
        .unwrap();
    let manager =
        ConnectionManager::<InstrumentedPgConnection>::new(pg_connstr);
    let pool = r2d2::Pool::builder().max_size(2).build(manager).unwrap();

    let user_ids_cache = InstrumentedRedisCache::new(
        RedisCacheBuilder::new("test_user_ids", Duration::from_secs(3600))
            .connection_string(redis_connstr)
            .build()
            .unwrap(),
        actix_demo::metrics::Metrics::new(prometheus::Registry::new()).cache,
    );

    {
        let mut conn = pool.get().unwrap();
        let migrations: FileBasedMigrations =
            FileBasedMigrations::find_migrations_directory().unwrap();
        conn.run_pending_migrations(migrations).unwrap();
        insert_new_user(
            NewUser {
                username: Username::parse_str(DEFAULT_USER).unwrap(),
                password: Password::parse_str(DEFAULT_USER).unwrap(),
                email: Email::try_from("admin@example.com".to_string())
                    .unwrap(),
            },
            RoleEnum::RoleAdmin,
            config.hash_cost,
            &user_ids_cache,
            &mut conn,
        )
        .unwrap();
    }

    build_app_data(
        pg_connstr,
        redis_connstr,
        minio_connstr,
        config,
        client,
        cm,
        pool,
        user_ids_cache,
    )
}

fn build_app_data(
    _pg_connstr: &str,
    _redis_connstr: &str,
    minio_connstr: &str,
    config: AppConfig,
    client: redis::Client,
    cm: redis::aio::ConnectionManager,
    pool: r2d2::Pool<ConnectionManager<InstrumentedPgConnection>>,
    user_ids_cache: InstrumentedRedisCache<
        String,
        Vec<actix_demo::models::users::UserId>,
    >,
) -> Data<AppData> {
    let redis_prefix = Box::new(|s: &dyn std::fmt::Display| {
        format!("test:{}:", s.to_string())
    });
    let credentials_repo = RedisCredentialsRepo::new(
        redis_prefix(&"user-sessions"),
        cm.clone(),
        10,
        300,
        actix_demo::metrics::Metrics::new(prometheus::Registry::new())
            .active_sessions,
    );

    let data = Data::new(AppData {
        start_time: std::time::SystemTime::now(),
        config,
        pool,
        credentials_repo,
        jwt_key: crate::common::TEST_JWT_KEY.clone(),
        redis_conn_factory: client.clone(),
        redis_conn_manager: cm.clone(),
        redis_prefix,
        sessions_cleanup_worker_handle: None,
        metrics: actix_demo::metrics::Metrics::new(prometheus::Registry::new()),
        prometheus: actix_web_prom::PrometheusMetricsBuilder::new("api")
            .endpoint("/metrics")
            .build()
            .unwrap(),
        user_ids_cache,
        health_checkers: Vec::new(),
        minio: minior::Minio {
            client: Arc::new(
                aws_sdk_s3::Client::from_conf(
                    aws_sdk_s3::config::Builder::new()
                        .endpoint_url(minio_connstr)
                        .credentials_provider(aws_sdk_s3::config::Credentials::new(
                            "minioadmin", "minioadmin", None, None, "testcontainers",
                        ))
                        .region(aws_sdk_s3::config::Region::new("test"))
                        .force_path_style(true)
                        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
                        .build(),
                ),
            ),
        },
        mailer: Arc::new(
            SmtpSender::new(&actix_demo::services::email::smtp::SmtpSenderConfig {
                smtp_host: "localhost".to_string(),
                smtp_port: 1025,
                tls_mode: TlsMode::None,
                username: "".to_string(),
                password: "".to_string(),
                from_email: "noreply@example.com".to_string(),
                verification_link_template: "https://app.example.com/verify?token={token}&user={user_name}".to_string(),
                password_reset_link_template: "https://app.example.com/reset?token={token}&user={user_name}".to_string(),
            })
            .unwrap(),
        ),
    });

    data
}
