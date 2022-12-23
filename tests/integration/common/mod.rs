extern crate actix_demo;
use actix_demo::actions::misc::create_database_if_needed;
use actix_demo::models::roles::RoleEnum;
use actix_demo::models::users::{NewUser, Password, Username};
use actix_demo::telemetry::DomainRootSpanBuilder;
use actix_demo::{utils, AppConfig, AppData};
use actix_web::dev::ServiceResponse;
use actix_web::test::TestRequest;
use actix_web::App;
use actix_web::{test, web};

use actix_web::web::Data;
use anyhow::Context;
use diesel::r2d2::{self, ConnectionManager};
use diesel_tracing::pg::InstrumentedPgConnection;
use jwt_simple::prelude::HS256Key;
use once_cell::sync::Lazy;
use rand::Rng;
use std::fs;
use std::io::{self, Write};
use std::os::unix::prelude::OpenOptionsExt;
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
use actix_test::TestServer;
use actix_web::body::MessageBody;
use actix_web::{dev::*, Error as AxError};
use lazy_static::lazy_static;
use std::sync::Arc;

pub const DEFAULT_USER: &str = "admin";

pub const BIN_FILE_LOCATION: &str = "/tmp/my-echo.sh";

pub const BIN_FILE_CONTENTS: &str = r#"#!/bin/bash

echo "hello world $1 $2";
"#;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    static ref PG: Container<'static, GenericImage> = DOCKER.run(
        GenericImage::new("postgres", "15-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
    );
    static ref REDIS: Container<'static, GenericImage> =
        DOCKER.run(GenericImage::new("redis", "7-alpine").with_wait_for(
            WaitFor::message_on_stdout("Ready to accept connections",)
        ));
    static ref REDIS_CONNSTR: String = {
        let port = REDIS.get_host_port_ipv4(6379);
        let connection_string = format!("redis://127.0.0.1:{port}");
        tracing::info!("Redis connstr={connection_string}");
        connection_string
    };
}

static TRACING: Lazy<anyhow::Result<()>> = Lazy::new(|| {
    let _ = dotenv::dotenv().context("Failed to set up env")?;
    let env_filter = EnvFilter::try_from_env("ACTIX_DEMO_TEST_RUST_LOG")
        .context("Failed to set up env logger")?;

    let _ = LogTracer::init()?;

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_test_writer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish()
        .with(env_filter);

    let _ =
        set_global_default(subscriber).context("Failed to set subscriber")?;
    Ok(())
});

static BIN_FILE: Lazy<anyhow::Result<()>> = Lazy::new(|| {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o777)
        .open(BIN_FILE_LOCATION)?;
    file.write_all(BIN_FILE_CONTENTS.as_bytes())?;
    file.flush()?;
    Ok(())
});

pub async fn app_data(connspec: &str) -> anyhow::Result<web::Data<AppData>> {
    let _ = Lazy::force(&TRACING).as_ref().unwrap();

    let _ = Lazy::force(&BIN_FILE).as_ref().unwrap();

    let config = AppConfig {
        hash_cost: 4,
        job_bin_path: BIN_FILE_LOCATION.to_string(),
    };

    let client = redis::Client::open(REDIS_CONNSTR.to_string())
        .context("failed to initialize redis")?;
    let cm = redis::aio::ConnectionManager::new(client.clone())
        .await
        .with_context(|| {
            let conn_string: String = REDIS_CONNSTR.to_string();
            format!("Failed to connect to redis. Url was: {conn_string}",)
        })?;

    let manager = ConnectionManager::<InstrumentedPgConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .max_size(2)
        .build(manager)
        .context("Failed to create pool")?;

    let _ = {
        let pool = pool.clone();
        let _ = web::block(move || {
            let conn = &pool.get()?;
            let migrations_dir = diesel_migrations::find_migrations_directory()
                .context("Error finding migrations dir")?;
            let _ = diesel_migrations::run_pending_migrations_in_directory(
                conn,
                &migrations_dir,
                &mut io::sink(),
            )
            .context("Error running migrations")?;
            actix_demo::actions::users::insert_new_user(
                NewUser {
                    username: Username::parse_str(DEFAULT_USER)?,
                    password: Password::parse_str(DEFAULT_USER)?,
                },
                RoleEnum::RoleAdmin,
                config.hash_cost,
                conn,
            )?;
            Ok::<(), anyhow::Error>(())
        })
        .await??;
    };

    let credentials_repo =
        Arc::new(actix_demo::utils::InMemoryCredentialsRepo::default());
    let key = HS256Key::from_bytes("test".as_bytes());

    let redis_prefix = {
        let mut rng = rand::thread_rng();
        let n1: u8 = rng.gen();
        Box::new(utils::get_redis_prefix(format!("redis{n1}")))
    };

    let data = Data::new(AppData {
        config,
        pool,
        credentials_repo,
        jwt_key: key,
        redis_conn_factory: Some(client.clone()),
        redis_conn_manager: Some(cm.clone()),
        redis_prefix,
    });
    Ok(data)
}

pub async fn test_app(
    connspec: &str,
) -> anyhow::Result<
    impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
> {
    let app = App::new()
        .configure(configure_app(app_data(connspec).await?))
        .wrap(TracingLogger::<DomainRootSpanBuilder>::new());
    let test_app = test::init_service(app).await;
    Ok(test_app)
}

pub async fn test_http_app(connspec: &str) -> anyhow::Result<TestServer> {
    let data = app_data(connspec).await?;
    let test_app = move || {
        App::new()
            .configure(configure_app(data.clone()))
            .wrap(TracingLogger::<DomainRootSpanBuilder>::new())
    };
    Ok(actix_test::start(test_app))
}

pub async fn create_user(
    username: &str,
    password: &str,
    test_app: &impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
) -> String {
    let req = test::TestRequest::post()
        .append_header(("content-type", "application/json"))
        .set_payload(format!(
            r#"{{"username":"{username}","password":"{password}"}}"#
        ))
        .uri("/api/registration")
        .to_request();
    let _ = test_app.call(req).await.unwrap();
    get_token(username, password, test_app).await
}

pub async fn get_token(
    username: &str,
    password: &str,
    test_app: &impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
) -> String {
    let req = test::TestRequest::post()
        .append_header(("content-type", "application/json"))
        .set_payload(format!(
            r#"{{"username":"{username}","password":"{password}"}}"#
        ))
        .uri("/api/login")
        .to_request();
    let resp: ServiceResponse<_> = test_app.call(req).await.unwrap();
    // let body: ApiResponse<String> = test::read_body_json(resp).await;
    // println!("{:?}", body);
    resp.headers()
        .get("X-AUTH-TOKEN")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub async fn get_default_token(
    test_app: &impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
) -> String {
    get_token(DEFAULT_USER, DEFAULT_USER, test_app).await
}

pub fn pg_conn_string() -> anyhow::Result<String> {
    let mut rng = rand::thread_rng();
    let n1: u8 = rng.gen();
    let db = format!("postgres{n1}");
    let port = PG.get_host_port_ipv4(5432);
    let connection_string =
        format!("postgres://postgres:postgres@127.0.0.1:{port}/{db}");
    let _ =
        create_database_if_needed(&connection_string).with_context(|| {
            format!(
                "Failed to create/detect database. URL was {connection_string}"
            )
        })?;

    Ok(connection_string)
}

pub trait WithToken {
    fn with_token(self, token: String) -> Self;
}

impl WithToken for TestRequest {
    fn with_token(self, token: String) -> Self {
        self.append_header(("Authorization", format! {"Bearer {}", token}))
    }
}
