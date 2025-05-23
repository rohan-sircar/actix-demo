#![forbid(unsafe_code)]
#![allow(clippy::let_unit_value)]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use actix_demo::actions::misc::create_database_if_needed;
use actix_demo::config::MinioConfig;
use actix_demo::health::create_health_checkers;
use actix_demo::models::rate_limit::{
    KeyStrategy, RateLimitConfig, RateLimitPolicy,
};
use actix_demo::models::session::{SessionConfig, SessionRenewalPolicy};
use actix_demo::models::worker::{WorkerBackoffConfig, WorkerConfig};
use actix_demo::utils::redis_credentials_repo::RedisCredentialsRepo;
use actix_demo::utils::InstrumentedRedisCache;
use actix_demo::{
    config::EnvConfig, utils, workers, AppConfig, AppData, LoggerFormat,
};
use actix_web::web::Data;
use actix_web_prom::PrometheusMetricsBuilder;
use anyhow::Context;
use cached::stores::RedisCacheBuilder;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use diesel_tracing::pg::InstrumentedPgConnection;
use jwt_simple::prelude::HS256Key;
use minior::aws_sdk_s3;
use minior::aws_sdk_s3::config::{Credentials, Region};
use reqwest::Client;
use tokio::task::JoinHandle;
use tracing::subscriber::set_global_default;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let start_time = SystemTime::now();
    let _ = dotenvy::dotenv().context("Failed to set up env")?;

    let env_config = envy::prefixed("ACTIX_DEMO_")
        .from_env::<EnvConfig>()
        .context("Failed to parse config")?;

    //bind guard to variable instead of _
    let _guard = setup_logger(
        env_config.logger_format.clone(),
        env_config.loki_url.clone(),
    )?;

    // tracing::error!("config: {:?}", env_config);

    let connspec = &env_config.database_url;
    let _ = create_database_if_needed(connspec).with_context(|| {
        format!("Failed to create/detect database. URL was {connspec}")
    })?;
    let manager = ConnectionManager::<InstrumentedPgConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .context("Failed to create db pool")?;

    let _ = {
        let mut conn = pool.get().context("Failed to get connection")?;

        let migrations: FileBasedMigrations =
            FileBasedMigrations::find_migrations_directory()
                .context("Error running migrations")?;
        let _ = conn
            .run_pending_migrations(migrations)
            .map_err(|e| anyhow::anyhow!(e)) // Convert error to anyhow::Error
            .context("Error running migrations")?;
    };

    let client = redis::Client::open(env_config.redis_url.clone())
        .context("failed to initialize redis")?;
    let cm = redis::aio::ConnectionManager::new(client.clone())
        .await
        .with_context(|| {
            format!(
                "Failed to connect to redis. Url was: {}",
                &env_config.redis_url
            )
        })?;

    let redis_prefix = Box::new(utils::get_redis_prefix("app"));

    let session_config = SessionConfig {
        expiration_secs: env_config.session_expiration_secs,
        renewal: SessionRenewalPolicy {
            enabled: env_config.session_renewal_enabled,
            renewal_window_secs: env_config.session_renewal_window_secs,
            max_renewals: env_config.session_max_renewals,
        },
        cleanup_interval_secs: env_config.session_cleanup_interval_secs,
        max_concurrent_sessions: env_config.max_concurrent_sessions,
        disable: env_config.session_disable,
    };

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to build prometheus metrics registry")?;

    let metrics =
        actix_demo::metrics::Metrics::new(prometheus.clone().registry);

    let credentials_repo = RedisCredentialsRepo::new(
        redis_prefix(&"user-sessions"),
        cm.clone(),
        session_config.max_concurrent_sessions,
        session_config.renewal.renewal_window_secs,
        metrics.active_sessions.clone(),
    );
    let jwt_key = HS256Key::from_bytes(env_config.jwt_key.as_bytes());

    let rate_limit_config = RateLimitConfig {
        key_strategy: KeyStrategy::Ip, // Default to IP-based rate limiting
        auth: RateLimitPolicy {
            max_requests: env_config.rate_limit_auth_max_requests,
            window_secs: env_config.rate_limit_auth_window_secs,
        },
        api: RateLimitPolicy {
            max_requests: env_config.rate_limit_api_max_requests,
            window_secs: env_config.rate_limit_api_window_secs,
        },
        api_public: RateLimitPolicy {
            max_requests: env_config.rate_limit_api_public_max_requests,
            window_secs: env_config.rate_limit_api_public_window_secs,
        },
        disable: env_config.rate_limit_disable,
    };

    let credentials_repo_clone = credentials_repo.clone();
    let pool_clone = pool.clone();

    let user_ids_cache = InstrumentedRedisCache::new(
        RedisCacheBuilder::new("user_ids", 3600)
            .set_connection_string(&env_config.redis_url)
            .build()
            .map_err(|e| {
                anyhow::anyhow!("Failed to build user_ids cache: {:?}", e)
            })?,
        metrics.cache.clone(),
    );

    let sessions_cleanup_worker_handle: JoinHandle<()> = {
        let config = WorkerConfig {
            backoff: WorkerBackoffConfig {
                initial_interval_secs: env_config.worker_initial_interval_secs,
                multiplier: env_config.worker_multiplier,
                max_interval_secs: env_config.worker_max_interval_secs,
                max_elapsed_time_secs: env_config.worker_max_elapsed_time_secs,
            },
            run_interval: env_config.session_cleanup_interval_secs,
        };
        workers::start_sessions_cleanup_worker(
            config,
            credentials_repo_clone,
            user_ids_cache.clone(),
            pool_clone,
        )
        .await
    };

    let http_client = Client::builder()
        .timeout(Duration::from_secs(
            env_config.health_check_timeout_secs.into(),
        ))
        .build()
        .context("Failed to create HTTP client")?;

    let cred = Credentials::new(
        &env_config.minio_access_key,
        &env_config.minio_secret_key,
        None,
        None,
        "loaded-from-custom-env",
    );

    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(&env_config.minio_endpoint)
        .credentials_provider(cred)
        .region(Region::new("custom-local")) // Custom region for self-hosted MinIO
        .force_path_style(true) // apply bucketname as path param instead of pre-domain
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .build();

    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

    // Ensure bucket exists
    let bucket_exists = s3_client
        .head_bucket()
        .bucket(&env_config.minio_bucket_name)
        .send()
        .await;

    if bucket_exists.is_err() {
        let _ =
            tracing::info!("Creating bucket {}", &env_config.minio_bucket_name);
        let _ = s3_client
            .create_bucket()
            .bucket(&env_config.minio_bucket_name)
            .send()
            .await
            .with_context(|| {
                format!(
                    "Failed to create bucket {}",
                    &env_config.minio_bucket_name
                )
            })?;
    }

    let minio = minior::Minio {
        client: Arc::new(s3_client),
    };

    let health_checkers = create_health_checkers(
        pool.clone(),
        cm.clone(),
        env_config.loki_url.clone(),
        env_config.prometheus_url.clone(),
        http_client,
    );

    let app_data = Data::new(AppData {
        start_time,
        config: AppConfig {
            hash_cost: env_config.hash_cost,
            job_bin_path: env_config.job_bin_path,
            rate_limit: rate_limit_config,
            session: session_config,
            health_check_timeout_secs: env_config.health_check_timeout_secs,
            minio: MinioConfig {
                bucket_name: env_config.minio_bucket_name,
                max_avatar_size_bytes: env_config.max_avatar_size_bytes,
            },
            timezone: env_config.timezone,
        },
        pool,
        credentials_repo,
        jwt_key,
        redis_conn_factory: client.clone(),
        redis_conn_manager: cm.clone(),
        redis_prefix,
        sessions_cleanup_worker_handle: Some(sessions_cleanup_worker_handle),
        metrics,
        prometheus,
        user_ids_cache,
        health_checkers,
        minio,
    });

    let _app =
        actix_demo::run(format!("{}:7800", env_config.http_host), app_data)
            .await?;

    Ok(())
}

pub fn setup_logger(
    format: LoggerFormat,
    loki_url: url::Url,
) -> anyhow::Result<(WorkerGuard, JoinHandle<()>)> {
    let env_filter = EnvFilter::try_from_env("ACTIX_DEMO_RUST_LOG")
        .context("Failed to set up env logger")?;

    let (non_blocking, _guard) =
        tracing_appender::non_blocking(std::io::stdout());

    let _ = LogTracer::init().context("Failed to set up log tracer")?;

    let bi = actix_demo::get_build_info();

    let (loki_layer, loki_task) = tracing_loki::builder()
        .label("host", "mine")?
        .extra_field("pid", format!("{}", std::process::id()))?
        .label("app", format!("actix-demo-{}", bi.crate_info.version))?
        .build_url(loki_url)?;

    let subscriber = Registry::default().with(env_filter).with(loki_layer);

    let _ = match format {
        LoggerFormat::Json => {
            let formatting_layer = BunyanFormattingLayer::new(
                format!("actix-demo-{}", bi.crate_info.version),
                non_blocking,
            );
            let subscriber =
                subscriber.with(JsonStorageLayer).with(formatting_layer);
            let _ = set_global_default(subscriber)
                .context("Failed to set subscriber")?;
        }

        LoggerFormat::Pretty => {
            let pretty_formatter = tracing_subscriber::fmt::Layer::new()
                .pretty()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(non_blocking)
                .with_thread_names(true);
            let subscriber = subscriber.with(pretty_formatter);
            let _ = set_global_default(subscriber)
                .context("Failed to set subscriber")?;
        }
    };

    let loki_guard = tokio::spawn(loki_task);
    Ok((_guard, loki_guard))
}
