#![forbid(unsafe_code)]
#![allow(clippy::let_unit_value)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate diesel_derive_newtype;

pub mod actions;
pub mod config;
pub mod errors;
pub mod health;
pub mod metrics;
pub mod middlewares;
pub mod models;
mod rate_limit;
mod routes;
pub mod schema;
pub mod services;
pub mod telemetry;
pub mod types;
pub mod utils;
pub mod workers;

use utoipa::OpenApi;

use std::sync::Arc;
use std::time::SystemTime;

use actix_cors::Cors;
use actix_web_prom::PrometheusMetrics;

use actix_web::http::header;
use actix_web::middleware::from_fn;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{
    http::StatusCode, middleware, web, App, HttpResponse, HttpServer,
};
use actix_web_grants::GrantsMiddleware;
use config::{MinioConfig, OAuthConfig, TlsMode};
use health::{HealthChecker, HealthcheckName};
use jwt_simple::prelude::HS256Key;
use metrics::Metrics;
use models::rate_limit::{RateLimitConfig, RateLimitPolicy};
use models::session::SessionConfig;
use models::users::UserId;
use redis::aio::ConnectionManager;
use redis::Client;
use serde::Deserialize;
use services::email::Mailer;
use telemetry::DomainRootSpanBuilder;
use tokio::task::JoinHandle;
use tracing_actix_web::TracingLogger;
use types::{DbPool, RedisPrefixFn};
use utils::redis_credentials_repo::RedisCredentialsRepo;
use utils::InstrumentedRedisCache;
use utoipa_swagger_ui::SwaggerUi;

build_info::build_info!(pub fn get_build_info);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LoggerFormat {
    Json,
    Pretty,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub tls_mode: TlsMode,
}

pub struct AppConfig {
    pub hash_cost: u32,
    pub job_bin_path: String,
    pub rate_limit: RateLimitConfig,
    pub session: SessionConfig,
    pub health_check_timeout_secs: u8,
    pub minio: MinioConfig,
    pub timezone: chrono_tz::Tz,
    pub smtp: SmtpConfig,
    pub email_token_ttl_verification_secs: u64,
    pub email_token_ttl_reset_secs: u64,
    pub oauth: OAuthConfig,
    pub cors_origins: String,
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
    pub minio: minior::Minio,
    pub mailer: Arc<dyn Mailer>,
    pub swagger_path: String,
}

pub fn configure_app(
    app_data: Data<AppData>,
) -> Box<dyn Fn(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        // Configure rate limiter for auth endpoints
        let auth_rate_limiter = {
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

        let swagger_path = app_data.swagger_path.clone();
        let swagger_path_for_swaggerui = format!("{}/{{_:.*}}", swagger_path);
        cfg.app_data(app_data.clone())
            .service(web::resource(&swagger_path).route(web::get().to(
                move || {
                    let redirect = format!("{}/", swagger_path);
                    async move {
                        HttpResponse::SeeOther()
                            .status(StatusCode::SEE_OTHER)
                            .insert_header((
                                header::LOCATION,
                                redirect.as_str(),
                            ))
                            .finish()
                    }
                },
            )))
            .service(
                SwaggerUi::new(swagger_path_for_swaggerui)
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .service(
                web::scope("/hc")
                    .wrap(in_memory_rate_limiter)
                    .route("", web::get().to(routes::healthcheck::healthcheck)),
            )
            .service(
                web::scope("/api/v1/auth")
                    .wrap(auth_rate_limiter.clone())
                    .service(
                        web::resource("/login")
                            .route(web::post().to(routes::auth::login)),
                    )
                    .service(
                        web::resource("/exchange")
                            .route(web::post().to(routes::auth::exchange)),
                    )
                    .service(
                        web::resource("/logout")
                            .route(web::post().to(routes::auth::logout)),
                    )
                    .service(
                        web::resource("/registration")
                            .route(web::post().to(routes::users::add_user)),
                    )
                    .service(
                        web::resource("/verify-email")
                            .route(web::post().to(routes::auth::verify_email)),
                    )
                    .service(web::resource("/resend-verification-email").route(
                        web::post().to(routes::auth::resend_verification_email),
                    ))
                    .service(web::resource("/password-reset-request").route(
                        web::post().to(routes::auth::request_password_reset),
                    ))
                    .service(web::resource("/password-reset-complete").route(
                        web::post().to(routes::auth::complete_password_reset),
                    ))
                    .service(
                        web::scope("/oauth")
                            .wrap(api_rate_limiter(
                                &app_data.config.rate_limit.api_public,
                            ))
                            .service(
                                web::scope("/github")
                                    .route(
                                        "/login",
                                        web::get()
                                            .to(routes::oauth::github_login),
                                    )
                                    .route(
                                        "/callback",
                                        web::get()
                                            .to(routes::oauth::github_callback),
                                    )
                                    .route(
                                        "/exchange",
                                        web::post()
                                            .to(routes::oauth::github_exchange),
                                    ),
                            )
                            .service(
                                web::scope("/google")
                                    .route(
                                        "/login",
                                        web::get()
                                            .to(routes::oauth::google_login),
                                    )
                                    .route(
                                        "/callback",
                                        web::get()
                                            .to(routes::oauth::google_callback),
                                    )
                                    .route(
                                        "/exchange",
                                        web::post()
                                            .to(routes::oauth::google_exchange),
                                    ),
                            ),
                    ),
            )
            .service(
                web::scope("/ws")
                    .wrap(api_rate_limiter(
                        &app_data.config.rate_limit.api_public,
                    ))
                    .route("", web::get().to(routes::ws::ws)),
            )
            // authenticated api (must come before /api/v1 to avoid prefix match)
            .service(
                web::scope("/api/v1/private")
                    .wrap(api_rate_limiter(&app_data.config.rate_limit.api))
                    .wrap(GrantsMiddleware::with_extractor(
                        routes::auth::extract,
                    ))
                    .wrap(middleware::Condition::new(
                        true, // Always enabled
                        middlewares::CustomHeaders::new(
                            app_data.config.timezone,
                        ),
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
                        web::scope("/avatars")
                            .route(
                                "",
                                web::put()
                                    .to(routes::users::upload_user_avatar),
                            )
                            .route(
                                "",
                                web::delete()
                                    .to(routes::users::delete_user_avatar),
                            ),
                    )
                    .service(
                        web::scope("/sessions")
                            .route(
                                "",
                                web::get().to(routes::auth::list_sessions),
                            )
                            .route(
                                "/{session_id}",
                                web::delete().to(routes::auth::revoke_session),
                            )
                            .route(
                                "/revoke-others",
                                web::post()
                                    .to(routes::auth::revoke_other_sessions),
                            ),
                    )
                    .service(
                        web::scope("/user")
                            .route(
                                "",
                                web::get().to(routes::users::get_my_profile),
                            )
                            .route(
                                "",
                                web::patch()
                                    .to(routes::users::update_my_profile),
                            )
                            .route(
                                "/profile",
                                web::get().to(routes::users::get_user_profile),
                            )
                            .route(
                                "/profile",
                                web::post()
                                    .to(routes::users::create_user_profile),
                            )
                            .route(
                                "/profile",
                                web::patch()
                                    .to(routes::users::update_user_profile),
                            )
                            .route(
                                "",
                                web::delete()
                                    .to(routes::users::delete_my_account),
                            )
                            .service(
                                web::scope("/pets")
                                    .route(
                                        "",
                                        web::post()
                                            .to(routes::pets::create_pet),
                                    )
                                    .route(
                                        "",
                                        web::get().to(routes::pets::list_pets),
                                    )
                                    .route(
                                        "/{pet_uuid}",
                                        web::get().to(routes::pets::get_pet),
                                    )
                                    .route(
                                        "/{pet_uuid}",
                                        web::patch()
                                            .to(routes::pets::update_pet),
                                    )
                                    .route(
                                        "/{pet_uuid}",
                                        web::delete()
                                            .to(routes::pets::delete_pet),
                                    )
                                    .route(
                                        "/{pet_uuid}/images",
                                        web::post()
                                            .to(routes::pets::upload_pet_image),
                                    )
                                    .route(
                                        "/{pet_uuid}/images",
                                        web::get()
                                            .to(routes::pets::list_pet_images),
                                    )
                                    .route(
                                        "/{pet_uuid}/images/{image_uuid}",
                                        web::delete()
                                            .to(routes::pets::delete_pet_image),
                                    )
                                    .route(
                                        "/{pet_uuid}/images/{image_uuid}",
                                        web::patch().to(
                                            routes::pets::set_primary_pet_image,
                                        ),
                                    ),
                            ),
                    )
                    .service(
                        web::scope("/admin").service(
                            web::scope("/users")
                                .route(
                                    "",
                                    web::get().to(routes::users::get_users),
                                )
                                .route(
                                    "/{user_id}",
                                    web::get().to(routes::users::get_user),
                                ),
                        ),
                    )
                    // discover endpoints
                    .route(
                        "/discover/next",
                        web::get().to(routes::discover::discover_next),
                    )
                    .route(
                        "/discover/pets",
                        web::get().to(routes::discover::discover_pets),
                    )
                    .route(
                        "/likes",
                        web::post().to(routes::discover::create_like),
                    )
                    .route(
                        "/messages",
                        web::post().to(routes::discover::stub_messages),
                    )
                    .route(
                        "/reports",
                        web::post().to(routes::discover::stub_reports),
                    )
                    // user profile endpoints (moved from public)
                    .route(
                        "/profiles/{user_id}",
                        web::get().to(routes::users::get_public_profile),
                    )
                    // pet endpoints (moved from public)
                    .route(
                        "/pets/traits",
                        web::get().to(routes::pets::get_traits),
                    )
                    .route(
                        "/pets/{pet_uuid}",
                        web::get().to(routes::pets::get_public_pet),
                    )
                    .route(
                        "/pets/{pet_uuid}/images",
                        web::get().to(routes::pets::get_pet_images),
                    )
                    .route(
                        "/pets/images/{image_uuid}",
                        web::get().to(routes::pets::get_public_pet_image),
                    )
                    .route(
                        "/pets/images/{image_uuid}/{variant}",
                        web::get().to(routes::pets::get_pet_image_variant),
                    ),
            )
            // public api
            .service(
                web::scope("/api/v1")
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
                    )
                    .route(
                        "/avatars/{user_id}",
                        web::get().to(routes::users::get_user_avatar),
                    ),
            );
    })
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::auth::login,
        routes::auth::logout,
        routes::auth::list_sessions,
        routes::auth::revoke_session,
        routes::auth::revoke_other_sessions,
        routes::auth::verify_email,
        routes::auth::resend_verification_email,
        routes::auth::request_password_reset,
        routes::auth::complete_password_reset,
        routes::users::get_user,
        routes::users::get_users,
        routes::users::add_user,
        routes::users::upload_user_avatar,
        routes::users::delete_user_avatar,
        routes::users::get_user_avatar,
        routes::users::get_my_profile,
        routes::users::update_my_profile,
        routes::users::get_user_profile,
        routes::users::create_user_profile,
        routes::users::update_user_profile,
        routes::users::get_public_profile,
        routes::users::delete_my_account,
        routes::pets::get_traits,
        routes::pets::get_public_pet,
        routes::pets::get_pet_images,
        routes::pets::create_pet,
        routes::pets::list_pets,
        routes::pets::get_pet,
        routes::pets::update_pet,
        routes::pets::delete_pet,
        routes::pets::upload_pet_image,
        routes::pets::list_pet_images,
        routes::pets::delete_pet_image,
        routes::pets::set_primary_pet_image,
        routes::pets::get_public_pet_image,
        routes::pets::get_pet_image_variant,
        routes::command::handle_run_command,
        routes::command::handle_get_job,
        routes::command::handle_get_job_metrics,
        routes::command::handle_abort_job,
        routes::oauth::github_login,
        routes::oauth::github_callback,
        routes::oauth::github_exchange,
        routes::oauth::google_login,
        routes::oauth::google_callback,
        routes::oauth::google_exchange,
        routes::auth::exchange,
        routes::healthcheck::healthcheck,
        routes::misc::build_info_req,
        routes::discover::discover_next,
        routes::discover::discover_pets,
        routes::discover::create_like,
        routes::discover::stub_messages,
        routes::discover::stub_reports,
    ),
    components(
        schemas(
            models::users::NewUser,
            models::users::UserLogin,
            models::users::UpdateUserProfile,
            models::users::User,
            models::users::UserWithRoles,
            models::users::OAuthProvider,
            models::misc::ErrorResponseString,
            routes::auth::VerifyEmailRequest,
            routes::auth::PasswordResetRequest,
            routes::auth::ResendVerificationRequest,
            routes::auth::PasswordResetCompleteRequest,
            routes::auth::AuthResponse,
            routes::auth::AuthUser,
            routes::oauth::OAuthExchangeRequest,
            routes::command::RunCommandRequest,
            models::misc::Job,
            models::misc::NewJob,
            models::misc::JobCount,
            services::oauth::models::GitHubOAuthUser,
            services::oauth::models::GitHubEmail,
            services::oauth::models::GitHubTokenResponse,
            services::oauth::models::GoogleOAuthUser,
            services::oauth::models::GoogleTokenResponse,
            routes::healthcheck::HealthCheckResponse,
            routes::healthcheck::ServiceStatus,
            routes::users::UploadAvatarRequest,
            models::misc::JobStatus,
            models::session::SessionInfo,
            models::users::Email,
            models::users::UserId,
            models::users::Username,
            models::users::Profile,
            models::users::PublicProfile,
            models::users::UpdateProfile,
            models::users::CreateProfile,
            models::roles::RoleEnum,
            models::pets::PetId,
            models::pets::PetUuid,
            models::pets::TraitId,
            models::pets::CreatePet,
            models::pets::UpdatePet,
            models::pets::PublicPet,
            models::pets::PublicPetOwner,
            models::pets::PetTrait,
            models::pets::PersonalityTrait,
            models::pets::PetGender,
            models::pets::ImageId,
            models::pets::PetImage,
            models::pets::PublicPetImage,
            models::pets::PetImageVariant,
            models::pets::UploadPetImageRequest,
            models::likes::LikeDirection,
            models::likes::LikeId,
            models::likes::CreateLike,
            models::likes::LikeResponse,
        ),
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "pets", description = "Pet profiles endpoints"),
        (name = "oauth", description = "OAuth 2.0 endpoints"),
        (name = "command", description = "Background job execution"),
        (name = "public", description = "Public endpoints"),
        (name = "discover", description = "Discovery and swipe endpoints"),
        (name = "likes", description = "Likes and matches endpoints"),
    ),
)]
pub struct ApiDoc;

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
    let cors_origins = app_data.config.cors_origins.clone();
    tracing::info!(cors_origins = %cors_origins, "CORS config loaded");
    let app = move || {
        let cors = if cors_origins == "*" {
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600)
        } else {
            let mut cors_mw = Cors::default()
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
                .max_age(3600);
            for origin in cors_origins.split(',') {
                let origin = origin.trim();
                if !origin.is_empty() {
                    cors_mw = cors_mw.allowed_origin(origin);
                }
            }
            cors_mw
        };
        App::new()
            .wrap(cors)
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
