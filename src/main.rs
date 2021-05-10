#![forbid(unsafe_code)]
use actix_demo::{AppConfig, AppData, EnvConfig, LoggerFormat};
use diesel::{r2d2::ConnectionManager, SqliteConnection};
use io::ErrorKind;
use std::io;
use tracing::subscriber::set_global_default;
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

    let _ = setup_logger(env_config.logger_format)?;

    let connspec = &env_config.database_url;
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
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

    let app_data = AppData {
        config: AppConfig {
            hash_cost: env_config.hash_cost,
        },
        pool: pool.clone(),
    };

    actix_demo::run(format!("{}:7800", env_config.http_host), app_data).await
}

pub fn setup_logger(format: LoggerFormat) -> io::Result<()> {
    let env_filter =
        EnvFilter::try_from_env("ACTIX_DEMO_RUST_LOG").map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to set up env filter: {:?}", err),
            )
        })?;

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
                // Output the formatted spans to stdout.
                std::io::stdout,
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

        LoggerFormat::Plain => {
            let subscriber = FmtSubscriber::builder()
                .pretty()
                .with_span_events(FmtSpan::NEW)
                .with_span_events(FmtSpan::CLOSE)
                .with_env_filter(env_filter)
                .finish();
            let _ = set_global_default(subscriber).map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to set subscriber: {:?}", err),
                )
            })?;
        }
    };
    Ok(())
}
