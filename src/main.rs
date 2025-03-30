#![forbid(unsafe_code)]
#![allow(clippy::let_unit_value)]

use actix_demo::actions::misc::create_database_if_needed;
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
use tokio::task::JoinHandle;
use tracing::subscriber::set_global_default;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{
    layer::SubscriberExt, EnvFilter, FmtSubscriber, Registry,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv().context("Failed to set up env")?;

    let env_config = envy::prefixed("ACTIX_DEMO_")
        .from_env::<EnvConfig>()
        .context("Failed to parse config")?;

    //bind guard to variable instead of _
    let _guard = setup_logger(env_config.clone().logger_format)?;

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
    );

    let sessions_cleanup_worker_handle: JoinHandle<()> = {
        let config = WorkerConfig {
            backoff: WorkerBackoffConfig {
                initial_interval_secs: env_config.worker_initial_interval_secs,
                multiplier: env_config.worker_multiplier,
                max_interval_secs: env_config.worker_max_interval_secs,
                max_elapsed_time_secs: env_config.worker_max_elapsed_time_secs,
            },
            run_interval: env_config.worker_run_interval_secs,
        };
        workers::start_sessions_cleanup_worker(
            config,
            credentials_repo_clone,
            user_ids_cache.clone(),
            pool_clone,
        )
        .await
    };

    let app_data = Data::new(AppData {
        config: AppConfig {
            hash_cost: env_config.hash_cost,
            job_bin_path: env_config.job_bin_path,
            rate_limit: rate_limit_config,
            session: session_config,
        },
        pool,
        credentials_repo,
        jwt_key,
        redis_conn_factory: Some(client.clone()),
        redis_conn_manager: Some(cm.clone()),
        redis_prefix,
        sessions_cleanup_worker_handle: Some(sessions_cleanup_worker_handle),
        metrics,
        prometheus,
        user_ids_cache,
    });

    let _app =
        actix_demo::run(format!("{}:7800", env_config.http_host), app_data)
            .await?;

    Ok(())
}

pub fn setup_logger(format: LoggerFormat) -> anyhow::Result<WorkerGuard> {
    let env_filter = EnvFilter::try_from_env("ACTIX_DEMO_RUST_LOG")
        .context("Failed to set up env logger")?;

    let (non_blocking, _guard) =
        tracing_appender::non_blocking(std::io::stdout());

    let _ = LogTracer::init().context("Failed to set up log tracer")?;

    let bi = actix_demo::get_build_info();

    let _ = match format {
        LoggerFormat::Json => {
            let formatting_layer = BunyanFormattingLayer::new(
                format!("actix-demo-{}", bi.crate_info.version),
                // Output the formatted spans to non-blocking writer
                non_blocking,
            );
            let subscriber = Registry::default()
                .with(env_filter)
                .with(JsonStorageLayer)
                .with(formatting_layer);
            let _ = set_global_default(subscriber)
                .context("Failed to set subscriber")?;
        }

        LoggerFormat::Pretty => {
            let subscriber = FmtSubscriber::builder()
                .pretty()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_env_filter(env_filter)
                .with_writer(non_blocking)
                .with_thread_names(true)
                .finish();
            let _ = set_global_default(subscriber)
                .context("Failed to set subscriber")?;
        }
    };
    Ok(_guard)
}
