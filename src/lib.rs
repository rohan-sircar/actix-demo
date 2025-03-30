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
pub mod metrics;
pub mod models;
mod rate_limit;
mod routes;
mod schema;
// mod services;
pub mod telemetry;
pub mod types;
pub mod utils;
pub mod workers;

use actix_web_prom::PrometheusMetrics;

use actix_web::middleware::from_fn;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{middleware, web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use errors::DomainError;
use jwt_simple::prelude::HS256Key;
use models::rate_limit::RateLimitConfig;
use models::session::SessionConfig;
use models::users::UserId;
use redis::aio::ConnectionManager;
use redis::Client;
use serde::Deserialize;
use telemetry::DomainRootSpanBuilder;
use tokio::task::JoinHandle;
use tracing_actix_web::TracingLogger;
use types::{DbPool, RedisPrefixFn};
use utils::redis_credentials_repo::RedisCredentialsRepo;
use utils::InstrumentedRedisCache;

build_info::build_info!(pub fn get_build_info);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LoggerFormat {
    Json,
    Pretty,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfig {
    // system
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
    // rate limit
    pub rate_limit_auth_max_requests: u32,
    #[serde(default = "models::defaults::default_rate_limit_auth_window_secs")]
    pub rate_limit_auth_window_secs: u64,
    #[serde(default = "models::defaults::default_rate_limit_api_max_requests")]
    pub rate_limit_api_max_requests: u32,
    #[serde(default = "models::defaults::default_rate_limit_api_window_secs")]
    pub rate_limit_api_window_secs: u64,
    pub rate_limit_disable: bool,
    // session
    #[serde(default = "models::defaults::default_session_expiration_secs")]
    pub session_expiration_secs: u64,
    #[serde(
        default = "models::defaults::default_session_cleanup_interval_secs"
    )]
    pub session_cleanup_interval_secs: u64,
    #[serde(default = "models::defaults::default_max_concurrent_sessions")]
    pub max_concurrent_sessions: usize,
    #[serde(default = "models::defaults::default_session_renewal_enabled")]
    pub session_renewal_enabled: bool,
    #[serde(default = "models::defaults::default_session_renewal_window_secs")]
    pub session_renewal_window_secs: u64,
    #[serde(default = "models::defaults::default_session_max_renewals")]
    pub session_max_renewals: u32,
    #[serde(default)]
    pub session_disable: bool,
    // worker
    #[serde(
        default = "models::defaults::default_worker_initial_interval_secs"
    )]
    pub worker_initial_interval_secs: u64,
    #[serde(default = "models::defaults::default_worker_multiplier")]
    pub worker_multiplier: f64,
    #[serde(default = "models::defaults::default_worker_max_interval_secs")]
    pub worker_max_interval_secs: u64,
    #[serde(
        default = "models::defaults::default_worker_max_elapsed_time_secs"
    )]
    pub worker_max_elapsed_time_secs: u64,
    #[serde(default = "models::defaults::default_worker_run_interval_secs")]
    pub worker_run_interval_secs: u8,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub hash_cost: u32,
    pub job_bin_path: String,
    pub rate_limit: RateLimitConfig,
    pub session: SessionConfig,
}

pub struct AppData {
    pub config: AppConfig,
    pub pool: DbPool,
    pub credentials_repo: RedisCredentialsRepo,
    pub jwt_key: HS256Key,
    pub redis_conn_factory: Option<Client>,
    pub redis_conn_manager: Option<ConnectionManager>,
    pub redis_prefix: RedisPrefixFn,
    pub sessions_cleanup_worker_handle: Option<JoinHandle<()>>,
    pub metrics: metrics::Metrics,
    pub prometheus: PrometheusMetrics,
    pub user_ids_cache: InstrumentedRedisCache<String, Vec<UserId>>,
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

pub fn configure_app(
    app_data: Data<AppData>,
) -> Box<dyn Fn(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        // Configure rate limiter for login endpoint
        let login_limiter = rate_limit::create_login_rate_limiter(&app_data);

        // Configure rate limiter for other endpoints
        let api_rate_limiter =
            || rate_limit::create_api_rate_limiter(&app_data);

        cfg.app_data(app_data.clone())
            .service(
                web::resource("/api/login")
                    .wrap(login_limiter.clone())
                    .route(web::post().to(routes::auth::login)),
            )
            .service(
                web::resource("/api/logout")
                    .wrap(api_rate_limiter())
                    .route(web::post().to(routes::auth::logout)),
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
            .service(
                web::scope("/api/public")
                    .wrap(api_rate_limiter())
                    .route(
                        "/build-info",
                        web::get().to(routes::misc::build_info_req),
                    )
                    .route(
                        "/metrics/cmd",
                        web::get().to(routes::command::handle_get_job_metrics),
                    ),
            )
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
                    )
                    .service(
                        web::scope("/sessions")
                            .route(
                                "",
                                web::get().to(routes::auth::list_sessions),
                            )
                            .route(
                                "/{token}",
                                web::delete().to(routes::auth::revoke_session),
                            )
                            .route(
                                "/revoke-others",
                                web::post()
                                    .to(routes::auth::revoke_other_sessions),
                            ),
                    ),
            );
    })
}

pub async fn run(addr: String, app_data: Data<AppData>) -> anyhow::Result<()> {
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
            .wrap(app_data.prometheus.clone())
            .configure(configure_app(app_data.clone()))
            .wrap(TracingLogger::<DomainRootSpanBuilder>::new())
    };
    HttpServer::new(app)
        .bind(addr)?
        .run()
        .await
        .map_err(|err| anyhow::anyhow!(err))
}
