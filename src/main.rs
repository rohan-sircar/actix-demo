#![forbid(unsafe_code)]
use actix_demo::{AppConfig, AppData, EnvConfig};
use diesel::{r2d2::ConnectionManager, SqliteConnection};
use env_logger::Env;
use io::ErrorKind;
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let _ = dotenv::dotenv().map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to set up env: {:?}", err),
        )
    })?;

    let _ = env_logger::try_init_from_env(
        Env::default().filter("ACTIX_DEMO_RUST_LOG"),
    )
    .map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to set up env logger: {:?}", err),
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
