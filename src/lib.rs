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
pub mod config;
pub mod health;
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

use std::time::SystemTime;

use actix_web_prom::PrometheusMetrics;

use actix_web::middleware::from_fn;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{middleware, web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use health::{HealthChecker, HealthcheckName};
use jwt_simple::prelude::HS256Key;
use metrics::Metrics;
use models::rate_limit::{RateLimitConfig, RateLimitPolicy};
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
pub struct AppConfig {
    pub hash_cost: u32,
    pub job_bin_path: String,
    pub rate_limit: RateLimitConfig,
    pub session: SessionConfig,
    pub health_check_timeout_secs: u8,
}

pub struct AppData {
    pub start_time: SystemTime,
    pub config: AppConfig,
    pub pool: DbPool,
    pub credentials_repo: RedisCredentialsRepo,
    pub jwt_key: HS256Key,
    pub redis_conn_factory: Client,
    pub redis_conn_manager: ConnectionManager,
    pub redis_prefix: RedisPrefixFn,
    pub sessions_cleanup_worker_handle: Option<JoinHandle<()>>,
    pub metrics: Metrics,
    pub prometheus: PrometheusMetrics,
    pub user_ids_cache: InstrumentedRedisCache<String, Vec<UserId>>,
    pub health_checkers: Vec<(HealthcheckName, HealthChecker)>,
}

pub fn configure_app(
    app_data: Data<AppData>,
) -> Box<dyn Fn(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        // Configure rate limiter for login endpoint
        let login_limiter = {
            let backend = rate_limit::initialize_rate_limit_backend(&app_data);
            rate_limit::create_login_rate_limiter(
                &app_data.config.rate_limit,
                backend,
            )
        };

        // Configure rate limiter for other endpoints
        let api_rate_limiter = |policy: &RateLimitPolicy| {
            let backend = rate_limit::initialize_rate_limit_backend(&app_data);
            rate_limit::create_api_rate_limiter(
                &app_data.config.rate_limit.key_strategy,
                policy,
                backend,
            )
        };

        let in_memory_rate_limiter = {
            let backend = rate_limit::initialize_hc_backend(
                !app_data.health_checkers.is_empty(),
            );
            rate_limit::create_hc_rate_limiter(
                &app_data.config.rate_limit,
                backend,
            )
        };

        cfg.app_data(app_data.clone())
            .service(
                web::scope("/hc")
                    .wrap(in_memory_rate_limiter)
                    .route("", web::get().to(routes::healthcheck::healthcheck)),
            )
            .service(
                web::resource("/api/login")
                    .wrap(login_limiter.clone())
                    .route(web::post().to(routes::auth::login)),
            )
            .service(
                web::resource("/api/logout")
                    .wrap(api_rate_limiter(
                        &app_data.config.rate_limit.api_public,
                    ))
                    .route(web::post().to(routes::auth::logout)),
            )
            .service(
                web::resource("/api/registration")
                    .wrap(api_rate_limiter(
                        &app_data.config.rate_limit.api_public,
                    ))
                    .route(web::post().to(routes::users::add_user)),
            )
            .service(
                web::scope("/ws")
                    .wrap(api_rate_limiter(
                        &app_data.config.rate_limit.api_public,
                    ))
                    .route("", web::get().to(routes::ws::ws)),
            )
            .service(
                web::scope("/api/public")
                    .wrap(api_rate_limiter(
                        &app_data.config.rate_limit.api_public,
                    ))
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
                    .wrap(api_rate_limiter(&app_data.config.rate_limit.api))
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
