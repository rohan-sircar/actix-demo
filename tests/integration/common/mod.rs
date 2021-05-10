extern crate actix_demo;
use actix_demo::{AppConfig, AppData, EnvConfig};
use actix_web::test;
use actix_web::App;
use diesel::SqliteConnection;

use diesel::r2d2::{self, ConnectionManager};
use std::io;
use std::io::ErrorKind;
use tracing::subscriber::set_global_default;
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::{format::FmtSpan, Subscriber as FmtSubscriber};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

use actix_demo::configure_app;

use actix_http::Request;
use actix_web::{dev as ax_dev, Error as AxError};

pub async fn test_app() -> io::Result<
    impl ax_dev::Service<
        Request = Request,
        Response = ax_dev::ServiceResponse<impl ax_dev::MessageBody>,
        Error = AxError,
    >,
> {
    let _ = dotenv::dotenv().map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to set up env: {:?}", err),
        )
    })?;

    let _ = envy::prefixed("ACTIX_DEMO_")
        .from_env::<EnvConfig>()
        .map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to parse config: {:?}", err),
            )
        })?;

    let env_filter =
        EnvFilter::try_from_env("ACTIX_DEMO_RUST_LOG").map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to set up env logger: {:?}", err),
            )
        })?;

    let _ = LogTracer::init().map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to set up log tracer: {:?}", err),
        )
    });

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_test_writer()
        .with_span_events(FmtSpan::NEW)
        .with_span_events(FmtSpan::CLOSE)
        .finish()
        .with(env_filter);

    let _ = set_global_default(subscriber).map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to set subscriber: {:?}", err),
        )
    });

    let connspec = ":memory:";
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

        let migrations_dir = diesel_migrations::find_migrations_directory()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Error finding migrations dir: {:?}", err),
                )
            })?;
        let _ = diesel_migrations::run_pending_migrations_in_directory(
            conn,
            &migrations_dir,
            &mut io::sink(),
        )
        .map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Error running migrations: {:?}", err),
            )
        })?;
    };

    Ok(test::init_service(
        App::new()
            .configure(configure_app(AppData {
                config: AppConfig { hash_cost: 8 },
                pool,
            }))
            .wrap(TracingLogger),
    )
    .await)
}
