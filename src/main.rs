#![forbid(unsafe_code)]
#![allow(clippy::let_unit_value)]
use actix_demo::{AppConfig, AppData, EnvConfig, LoggerFormat};
use actix_web::web::Data;
use diesel::r2d2::ConnectionManager;
use diesel_tracing::sqlite::InstrumentedSqliteConnection;
use io::ErrorKind;
use jwt_simple::prelude::HS256Key;
use std::io;
use std::sync::Arc;
use tracing::subscriber::set_global_default;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{
    layer::SubscriberExt, EnvFilter, FmtSubscriber, Registry,
};

#[actix_web::main]
async fn main() -> io::Result<()> {
    let _ = dotenv::dotenv().map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to set up env: {:?}", err),
        )
    })?;

    let env_config = envy::prefixed("ACTIX_DEMO_")
        .from_env::<EnvConfig>()
        .map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to parse config: {:?}", err),
            )
        })?;

    //bind guard to variable instead of _
    let _guard = setup_logger(env_config.logger_format)?;

    let connspec = &env_config.database_url;
    let manager =
        ConnectionManager::<InstrumentedSqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder().build(manager).map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to create pool: {:?}", err),
        )
    })?;

    let _ = {
        let conn = &pool.get().map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to get connection: {:?}", err),
            )
        })?;

        let _ =
            diesel_migrations::run_pending_migrations(conn).map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Error running migrations: {:?}", err),
                )
            })?;
    };

    let credentials_repo =
        Arc::new(actix_demo::utils::InMemoryCredentialsRepo::default());
    let key = HS256Key::from_bytes(env_config.jwt_key.as_bytes());

    let app_data = Data::new(AppData {
        config: AppConfig {
            hash_cost: env_config.hash_cost,
        },
        pool: pool.clone(),
        credentials_repo,
        jwt_key: key,
    });

    actix_demo::run(format!("{}:7800", env_config.http_host), app_data).await
}

pub fn setup_logger(format: LoggerFormat) -> io::Result<WorkerGuard> {
    let env_filter =
        EnvFilter::try_from_env("ACTIX_DEMO_RUST_LOG").map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to set up env filter: {:?}", err),
            )
        })?;

    let (non_blocking, _guard) =
        tracing_appender::non_blocking(std::io::stdout());

    let _ = LogTracer::init().map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to set up log tracer: {:?}", err),
        )
    })?;

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
            let _ = set_global_default(subscriber).map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to set subscriber: {:?}", err),
                )
            })?;
        }

        LoggerFormat::Pretty => {
            let subscriber = FmtSubscriber::builder()
                .pretty()
                .with_span_events(FmtSpan::NEW)
                .with_span_events(FmtSpan::CLOSE)
                .with_env_filter(env_filter)
                .with_writer(non_blocking)
                .with_thread_names(true)
                .finish();
            let _ = set_global_default(subscriber).map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to set subscriber: {:?}", err),
                )
            })?;
        }
    };
    Ok(_guard)
}
