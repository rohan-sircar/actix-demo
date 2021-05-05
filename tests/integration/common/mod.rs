extern crate actix_demo;
use actix_demo::{AppConfig, AppData};
use actix_web::test;
use actix_web::App;
use diesel::SqliteConnection;

use diesel::r2d2::{self, ConnectionManager};
use env_logger::Env;
use std::io;
use std::io::ErrorKind;

use actix_demo::configure_app;

use actix_http::Request;
use actix_web::{dev as ax_dev, Error as AxError};

pub async fn test_app() -> impl ax_dev::Service<
    Request = Request,
    Response = ax_dev::ServiceResponse<impl ax_dev::MessageBody>,
    Error = AxError,
> {
    let _ = dotenv::dotenv()
        .map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to set up env: {:?}", err),
            )
        })
        .unwrap();
    let _ = env_logger::builder()
        .is_test(true)
        .parse_env(Env::default().filter("ACTIX_DEMO_TEST_RUST_LOG"))
        .try_init();

    let connspec = ":memory:";
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to create pool: {:?}", err),
            )
        })
        .unwrap();

    let _ = {
        let conn = &pool
            .get()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to get connection: {:?}", err),
                )
            })
            .unwrap();

        let migrations_dir = diesel_migrations::find_migrations_directory()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Error finding migrations dir: {:?}", err),
                )
            })
            .unwrap();
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
        })
        .unwrap();
    };

    test::init_service(
        App::new()
            .configure(configure_app(AppData {
                config: AppConfig { hash_cost: 8 },
                pool,
            }))
            .wrap(actix_web::middleware::Logger::default()),
    )
    .await
}
