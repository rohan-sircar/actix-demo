#![forbid(unsafe_code)]
#![allow(clippy::let_unit_value)]
#![allow(deprecated)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate derive_new;
// #[macro_use]
// extern crate validators_derive;
#[macro_use]
extern crate diesel_derive_newtype;

pub mod actions;
pub mod errors;
mod middlewares;
pub mod models;
mod routes;
mod schema;
mod services;
mod telemetry;
pub mod types;
pub mod utils;

use actix_files as fs;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use actix_web_httpauth::middleware::HttpAuthentication;
use jwt_simple::prelude::HS256Key;
use redis::aio::ConnectionManager;
use redis::Client;
use routes::auth::bearer_auth;
use serde::Deserialize;
use std::fmt::Display;
use std::io;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use utils::CredentialsRepo;

use types::DbPool;

use crate::telemetry::DomainRootSpanBuilder;

build_info::build_info!(pub fn get_build_info);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LoggerFormat {
    Json,
    Pretty,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfig {
    pub database_url: String,
    pub http_host: String,
    #[serde(default = "default_hash_cost")]
    pub hash_cost: u32,
    pub logger_format: LoggerFormat,
    pub jwt_key: String,
    pub redis_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub hash_cost: u32,
}

type RedisPrefixFn = Box<dyn Fn(&dyn Display) -> String + Send + Sync>;

pub struct AppData {
    pub config: AppConfig,
    pub pool: DbPool,
    pub credentials_repo: Arc<dyn CredentialsRepo + Send + Sync>,
    pub jwt_key: HS256Key,
    pub redis_conn_factory: Option<Client>,
    pub redis_conn_manager: Option<ConnectionManager>,
    pub redis_prefix: RedisPrefixFn,
}

pub fn default_hash_cost() -> u32 {
    8
}

pub fn configure_app(
    app_data: Data<AppData>,
) -> Box<dyn Fn(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        cfg.app_data(app_data.clone())
            .service(routes::auth::login)
            .service(routes::users::add_user)
            .service(web::scope("/ws").route("", web::get().to(routes::ws::ws)))
            // .service(routes::auth::logout)
            // public endpoint - not implemented yet
            .service(
                web::scope("/api/public")
                    .route(
                        "/build-info",
                        web::get().to(routes::misc::build_info_req),
                    )
                    .route("/cmd", web::post().to(routes::command::run_command))
                    .route(
                        "/cmd",
                        web::delete().to(routes::command::abort_command),
                    ),
            )
            .service(
                web::scope("/api")
                    .wrap(HttpAuthentication::bearer(bearer_auth))
                    .wrap(GrantsMiddleware::with_extractor(
                        routes::auth::extract,
                    ))
                    .service(
                        web::scope("/users")
                            .route("", web::get().to(routes::users::get_users))
                            .route(
                                "/search",
                                web::get().to(routes::users::search_users),
                            )
                            .route(
                                "/{user_id}",
                                web::get().to(routes::users::get_user),
                            ),
                    ),
            )
            .service(fs::Files::new("/", "./static").index_file("index.html"));
    })
}

pub async fn run(addr: String, app_data: Data<AppData>) -> io::Result<()> {
    let bi = get_build_info();
    let _ = tracing::info!(
        "Starting {} {}",
        bi.crate_info.name,
        bi.crate_info.version
    );
    println!(
        r#"
                       __  .__                     .___                     
        _____    _____/  |_|__|__  ___           __| _/____   _____   ____  
        \__  \ _/ ___\   __\  \  \/  /  ______  / __ |/ __ \ /     \ /  _ \ 
         / __ \\  \___|  | |  |>    <  /_____/ / /_/ \  ___/|  Y Y  (  <_> )
        (____  /\___  >__| |__/__/\_ \         \____ |\___  >__|_|  /\____/ 
             \/     \/              \/              \/    \/      \/        
         "#
    );
    let app = move || {
        App::new()
            .configure(configure_app(app_data.clone()))
            .wrap(TracingLogger::<DomainRootSpanBuilder>::new())
    };
    HttpServer::new(app).bind(addr)?.run().await
}
