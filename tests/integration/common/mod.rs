extern crate actix_demo;
use actix_demo::models::users::{NewUser, Password, Username};
use actix_demo::{AppConfig, AppData, EnvConfig};
use actix_http::header::HeaderValue;
use actix_web::App;
use actix_web::{test, web};

use actix_web::web::Data;
use diesel::r2d2::{self, ConnectionManager};
use jwt_simple::prelude::HS256Key;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use testcontainers::core::WaitFor;
use testcontainers::images::generic::GenericImage;
use testcontainers::*;
use tracing::subscriber::set_global_default;
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::{format::FmtSpan, Subscriber as FmtSubscriber};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use validators::prelude::*;

use actix_demo::configure_app;

use actix_http::Request;
use actix_web::{dev as ax_dev, Error as AxError};

pub async fn test_app(
    connspec: &str,
) -> io::Result<
    impl ax_dev::Service<
        Request,
        Response = ax_dev::ServiceResponse<impl actix_web::body::MessageBody>,
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

    let manager = ConnectionManager::<
        diesel_tracing::pg::InstrumentedPgConnection,
    >::new(connspec);
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

    let _ = {
        let pool = pool.clone();
        let _ = web::block(move || {
            let conn = &pool.get()?;
            actix_demo::actions::users::insert_new_user(
                NewUser {
                    username: Username::parse_str("user1").unwrap(),
                    password: Password::parse_str("test").unwrap(),
                },
                conn,
                8,
            )
        })
        .await
        .unwrap()
        .unwrap();
    };

    let credentials_repo =
        Arc::new(actix_demo::utils::InMemoryCredentialsRepo::default());
    let key = HS256Key::from_bytes("test".as_bytes());

    let test_app = test::init_service(
        App::new()
            .configure(configure_app(Data::new(AppData {
                config: AppConfig { hash_cost: 8 },
                pool,
                credentials_repo,
                jwt_key: key,
            })))
            .wrap(TracingLogger::default()),
    )
    .await;
    Ok(test_app)
}

pub async fn get_token(
    test_app: &impl ax_dev::Service<
        Request,
        Response = ax_dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = AxError,
    >,
) -> HeaderValue {
    let req = test::TestRequest::post()
        .append_header(("content-type", "application/json"))
        .set_payload(r#"{"username":"user1","password":"test"}"#)
        .uri("/api/login")
        .to_request();
    let resp: ax_dev::ServiceResponse<_> = test_app.call(req).await.unwrap();
    // let body: ApiResponse<String> = test::read_body_json(resp).await;
    // println!("{:?}", body);
    resp.headers().get("X-AUTH-TOKEN").unwrap().clone()
}

pub fn start_pg_container(
    docker: &'_ clients::Cli,
) -> (String, u16, Container<'_, GenericImage>) {
    let db = "postgres-db-test";
    let user = "postgres-user-test";
    let password = "postgres-password-test";

    let generic_postgres =
        images::generic::GenericImage::new("postgres", "15-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", db)
            .with_env_var("POSTGRES_USER", user)
            .with_env_var("POSTGRES_PASSWORD", password);
    // .with_exposed_port(port);

    let node = docker.run(generic_postgres);
    let port = node.get_host_port_ipv4(5432);

    let connection_string =
        format!("postgres://{}:{}@127.0.0.1:{}/{}", user, password, port, db);

    (connection_string, port, node)
}
