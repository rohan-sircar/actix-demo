#![forbid(unsafe_code)]
#![allow(clippy::let_unit_value)]
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
// mod middlewares;
pub mod models;
mod routes;
mod schema;
// mod services;
pub mod telemetry;
pub mod types;
pub mod utils;

use actix_extensible_rate_limit::HeaderCompatibleOutput;
use actix_extensible_rate_limit::{
    backend::{redis::RedisBackend, SimpleInputFunctionBuilder, SimpleOutput},
    RateLimiter,
};
use actix_files as fs;
use actix_http::header::{HeaderName, HeaderValue, RETRY_AFTER};

use actix_web::middleware::from_fn;
use actix_web::{middleware, web, App, HttpServer};
use actix_web::{
    web::{Data, ServiceConfig},
    HttpResponse,
};
use actix_web_grants::GrantsMiddleware;
use errors::DomainError;
use jwt_simple::prelude::HS256Key;
use models::rate_limit::{KeyStrategy, RateLimitConfig};
use rand::distr::Alphanumeric;
use rand::Rng;
use redis::aio::ConnectionManager;
use redis::Client;
use serde::Deserialize;
use std::io;
use telemetry::DomainRootSpanBuilder;
use tracing_actix_web::TracingLogger;
use types::{DbPool, RedisPrefixFn};
use utils::redis_credentials_repo::RedisCredentialsRepo;

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
    #[serde(default = "models::defaults::default_hash_cost")]
    pub hash_cost: u32,
    pub logger_format: LoggerFormat,
    pub jwt_key: String,
    pub redis_url: String,
    pub job_bin_path: String,
    #[serde(
        default = "models::defaults::default_rate_limit_auth_max_requests"
    )]
    pub rate_limit_auth_max_requests: u32,
    #[serde(default = "models::defaults::default_rate_limit_auth_window_secs")]
    pub rate_limit_auth_window_secs: u64,
    #[serde(default = "models::defaults::default_rate_limit_api_max_requests")]
    pub rate_limit_api_max_requests: u32,
    #[serde(default = "models::defaults::default_rate_limit_api_window_secs")]
    pub rate_limit_api_window_secs: u64,
    pub rate_limit_disable: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub hash_cost: u32,
    pub job_bin_path: String,
    pub rate_limit: RateLimitConfig,
}

pub struct AppData {
    pub config: AppConfig,
    pub pool: DbPool,
    pub credentials_repo: RedisCredentialsRepo,
    pub jwt_key: HS256Key,
    pub redis_conn_factory: Option<Client>,
    pub redis_conn_manager: Option<ConnectionManager>,
    pub redis_prefix: RedisPrefixFn,
}

impl AppData {
    pub fn get_redis_conn(&self) -> Result<ConnectionManager, DomainError> {
        self.redis_conn_manager.clone().ok_or_else(|| {
            DomainError::new_internal_error(
                "Redis connection not initialized".to_owned(),
            )
        })
    }
}

fn build_input_function(
    app_data: &AppData,
    input_fn_builder: SimpleInputFunctionBuilder,
) -> SimpleInputFunctionBuilder {
    match app_data.config.rate_limit.key_strategy {
        KeyStrategy::Ip => input_fn_builder.real_ip_key(),
        KeyStrategy::Random => {
            let random_suffix: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();
            let unique_key = format!("{}-{}", "test", random_suffix);
            input_fn_builder.custom_key(&unique_key)
        }
    }
}

#[allow(clippy::declare_interior_mutable_const)]
pub const X_RATELIMIT_LIMIT: HeaderName =
    HeaderName::from_static("x-ratelimit-limit");
#[allow(clippy::declare_interior_mutable_const)]
pub const X_RATELIMIT_REMAINING: HeaderName =
    HeaderName::from_static("x-ratelimit-remaining");
#[allow(clippy::declare_interior_mutable_const)]
pub const X_RATELIMIT_RESET: HeaderName =
    HeaderName::from_static("x-ratelimit-reset");

fn make_denied_response(status: &SimpleOutput) -> HttpResponse {
    let mut response = HttpResponse::TooManyRequests().finish();
    let map = response.headers_mut();
    map.insert(X_RATELIMIT_LIMIT, HeaderValue::from(status.limit()));
    map.insert(X_RATELIMIT_REMAINING, HeaderValue::from(status.remaining()));
    let seconds: u64 = status.seconds_until_reset();
    map.insert(X_RATELIMIT_RESET, HeaderValue::from(seconds));
    map.insert(RETRY_AFTER, HeaderValue::from(seconds));
    response
}

pub fn configure_app(
    app_data: Data<AppData>,
) -> Box<dyn Fn(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        // Configure rate limiter for login endpoint
        let login_limiter = {
            let input_fn_builder = SimpleInputFunctionBuilder::new(
                std::time::Duration::from_secs(
                    app_data.config.rate_limit.auth.window_secs,
                ),
                app_data.config.rate_limit.auth.max_requests.into(),
            );
            let input_fn =
                build_input_function(&app_data, input_fn_builder).build();

            let backend = if app_data.config.rate_limit.disable {
                utils::RateLimitBackend::Noop
            } else {
                let redis_cm = app_data
                    .get_redis_conn()
                    .expect("Redis connection required for rate limiting");
                utils::RateLimitBackend::Redis(
                    RedisBackend::builder(redis_cm).build(),
                )
            };

            RateLimiter::builder(backend, input_fn)
                .rollback_condition(Some(|status| {
                    status != actix_web::http::StatusCode::UNAUTHORIZED
                }))
                .add_headers()
                .request_denied_response(|status| {
                    let _ = tracing::warn!("Reached rate limit for login");
                    make_denied_response(status)
                })
                .build()
        };

        // Configure rate limiter for other endpoints
        let api_rate_limiter = || {
            let backend = if app_data.config.rate_limit.disable {
                utils::RateLimitBackend::Noop
            } else {
                let redis_cm = app_data
                    .get_redis_conn()
                    .expect("Redis connection required for rate limiting");
                utils::RateLimitBackend::Redis(
                    RedisBackend::builder(redis_cm).build(),
                )
            };

            let input_fn_builder = SimpleInputFunctionBuilder::new(
                std::time::Duration::from_secs(
                    app_data.config.rate_limit.api.window_secs,
                ),
                app_data.config.rate_limit.api.max_requests.into(),
            );
            let input_fn =
                build_input_function(&app_data, input_fn_builder).build();
            RateLimiter::builder(backend, input_fn)
                .add_headers()
                .request_denied_response(|status| {
                    let _ = tracing::warn!("Reached rate limit for api");
                    make_denied_response(status)
                })
                .build()
        };

        cfg.app_data(app_data.clone())
            .service(
                web::resource("/api/login")
                    .wrap(login_limiter)
                    .route(web::post().to(routes::auth::login)), // reference the function directly
            )
            .service(
                web::resource("/api/registration")
                    .wrap(api_rate_limiter())
                    .route(web::post().to(routes::users::add_user)),
            )
            .service(
                web::scope("/ws")
                    .wrap(api_rate_limiter())
                    .route("", web::get().to(routes::ws::ws)),
            )
            // TODO Implement logout
            // .service(routes::auth::logout)
            // public endpoint - not implemented yet
            .service(web::scope("/api/public").wrap(api_rate_limiter()).route(
                "/build-info",
                web::get().to(routes::misc::build_info_req),
            ))
            .service(
                web::scope("/api")
                    .wrap(api_rate_limiter())
                    .wrap(GrantsMiddleware::with_extractor(
                        routes::auth::extract,
                    ))
                    .wrap(middleware::Condition::new(
                        true, // Always enabled
                        middleware::DefaultHeaders::new()
                            .add(("Vary", "Cookie")),
                    ))
                    .wrap(from_fn(utils::cookie_auth))
                    .route(
                        "/cmd",
                        web::post().to(routes::command::handle_run_command),
                    )
                    .route(
                        "/cmd/{job_id}",
                        web::get().to(routes::command::handle_get_job),
                    )
                    .route(
                        "/cmd/{job_id}",
                        web::delete().to(routes::command::handle_abort_job),
                    )
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
