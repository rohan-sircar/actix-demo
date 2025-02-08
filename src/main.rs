#![forbid(unsafe_code)]
#![allow(clippy::let_unit_value)]
use actix_demo::actions::misc::create_database_if_needed;
use actix_demo::utils::redis_credentials_repo::RedisCredentialsRepo;
use actix_demo::{utils, AppConfig, AppData, EnvConfig, LoggerFormat};
use actix_web::web::Data;
use anyhow::Context;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use diesel_tracing::pg::InstrumentedPgConnection;
use jwt_simple::prelude::HS256Key;
use std::sync::Arc;
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
    let _ = dotenv::dotenv().context("Failed to set up env")?;

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

    let credentials_repo = Arc::new(RedisCredentialsRepo::new(
        redis_prefix(&"user-sessions"),
        cm.clone(),
    ));
    let jwt_key = HS256Key::from_bytes(env_config.jwt_key.as_bytes());

    let app_data = Data::new(AppData {
        config: AppConfig {
            hash_cost: env_config.hash_cost,
            job_bin_path: env_config.job_bin_path,
        },
        pool,
        credentials_repo,
        jwt_key,
        redis_conn_factory: Some(client.clone()),
        redis_conn_manager: Some(cm.clone()),
        redis_prefix,
    });

    Ok(
        actix_demo::run(format!("{}:7800", env_config.http_host), app_data)
            .await?,
    )
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
