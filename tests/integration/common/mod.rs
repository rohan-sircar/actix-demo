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
use derive_builder::Builder;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use diesel_tracing::pg::InstrumentedPgConnection;
use jwt_simple::prelude::HS256Key;
use once_cell::sync::Lazy;
use rand::Rng;
use std::fs;
use std::io::Write;
use std::os::unix::prelude::OpenOptionsExt;
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
use std::sync::Arc;

pub const DEFAULT_USER: &str = "admin";

fn redis_connstr() -> String {
    let port = 5556;
    let connection_string = format!("redis://127.0.0.1:{port}");
    tracing::info!("Redis connstr={connection_string}");
    connection_string
}

#[derive(Clone, Debug)]
pub struct BinFile {
    pub location: String,
    pub contents: String,
}

pub fn echo_bin_file() -> BinFile {
    BinFile {
        location: "/tmp/my-echo.sh".to_owned(),
        contents: r#"#!/bin/bash

echo "hello world $1 $2";
"#
        .to_owned(),
    }
}
pub fn sleep_bin_file() -> BinFile {
    BinFile {
        location: "/tmp/sleeper.sh".to_owned(),
        contents: r#"#!/bin/bash
    
    echo "sleeping"
    for i in {1..5}
    do
        echo "$i still sleeping"
        sleep 2
    done
    echo "done sleeping"
    "#
        .to_owned(),
    }
}

pub fn failing_bin_file() -> BinFile {
    BinFile {
        location: "/tmp/failing.sh".to_owned(),
        contents: r#"#!/bin/bash
    
    echo "I'm a failing script"
    exit 1
    "#
        .to_owned(),
    }
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

static CREATE_BIN_FILES: Lazy<anyhow::Result<()>> = Lazy::new(|| {
    let file1 = echo_bin_file();
    let file2 = sleep_bin_file();
    let file3 = failing_bin_file();
    let files = vec![file1, file2, file3];
    for f in &files {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o777)
            .open(&f.location)?;
        file.write_all(f.contents.as_bytes())?;
        file.flush()?;
    }
    Ok(())
});

#[derive(Clone, Builder, Debug)]
pub struct TestAppOptions {
    #[builder(default = "self.default_bin_file()")]
    pub bin_file: BinFile,
}

impl Default for TestAppOptions {
    fn default() -> Self {
        TestAppOptionsBuilder::default().build().unwrap()
    }
}

impl TestAppOptionsBuilder {
    fn default_bin_file(&self) -> BinFile {
        echo_bin_file()
    }
}

pub async fn app_data(
    connspec: &str,
    options: TestAppOptions,
) -> anyhow::Result<web::Data<AppData>> {
    let _ = Lazy::force(&TRACING).as_ref().unwrap();

    let _ = Lazy::force(&CREATE_BIN_FILES).as_ref().unwrap();

    let config = AppConfig {
        hash_cost: 4,
        job_bin_path: options.bin_file.location.clone(),
    };

    let client = redis::Client::open(redis_connstr())
        .context("failed to initialize redis")?;
    let cm = redis::aio::ConnectionManager::new(client.clone())
        .await
        .with_context(|| {
            let conn_string: String = redis_connstr();
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
            let _ = {
                let mut conn =
                    pool.get().context("Failed to get connection")?;

                let migrations: FileBasedMigrations =
                    FileBasedMigrations::find_migrations_directory()
                        .context("Error running migrations")?;
                let _ = conn
                    .run_pending_migrations(migrations)
                    .map_err(|e| anyhow::anyhow!(e)) // Convert error to anyhow::Error
                    .context("Error running migrations")?;
                actix_demo::actions::users::insert_new_user(
                    NewUser {
                        username: Username::parse_str(DEFAULT_USER)?,
                        password: Password::parse_str(DEFAULT_USER)?,
                    },
                    RoleEnum::RoleAdmin,
                    config.hash_cost,
                    &mut conn,
                )?;
            };

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
    options: TestAppOptions,
) -> anyhow::Result<
    impl Service<
        Request,
        Response = ServiceResponse<impl MessageBody>,
        Error = AxError,
    >,
> {
    let app = App::new()
        .configure(configure_app(app_data(connspec, options).await?))
        .wrap(TracingLogger::<DomainRootSpanBuilder>::new());
    let test_app = test::init_service(app).await;
    Ok(test_app)
}

pub async fn test_http_app(
    connspec: &str,
    options: TestAppOptions,
) -> anyhow::Result<TestServer> {
    let data = app_data(connspec, options).await?;
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
    let port = 5555;
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
