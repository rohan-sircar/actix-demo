#![forbid(unsafe_code)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate log;

mod actions;
mod errors;
mod middlewares;
pub mod models;
mod routes;
mod schema;
mod services;
mod types;
mod utils;

use actix_files as fs;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{cookie::SameSite, middleware, web, App, HttpServer};
use actix_web::{middleware::Logger, web::ServiceConfig};
use rand::Rng;
use serde::Deserialize;
use types::DbPool;

build_info::build_info!(pub fn get_build_info);

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfig {
    pub database_url: String,
    pub http_host: String,
    #[serde(default = "default_hash_cost")]
    pub hash_cost: u8,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub hash_cost: u8,
}

#[derive(Clone)]
pub struct AppData {
    pub config: AppConfig,
    pub pool: DbPool,
}

pub fn default_hash_cost() -> u8 {
    8
}

pub fn configure_app(app_data: AppData) -> Box<dyn Fn(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        cfg.data(app_data.clone())
            .service(
                web::scope("/api")
                    .service(routes::users::get_user)
                    .service(routes::users::get_all_users)
                    .service(web::scope("/get").route(
                        "/build-info",
                        web::get().to(routes::misc::build_info_req),
                    )),
            )
            // .route("/api/users/get", web::get().to(user_controller.get_user.into()))
            .service(web::scope("/api/public")) // public endpoint - not implemented yet
            .service(routes::auth::login)
            .service(routes::auth::logout)
            .service(routes::auth::index)
            .service(routes::users::add_user)
            .service(fs::Files::new("/", "./static"));
    })
}

pub fn id_service(
    private_key: &[u8],
) -> actix_identity::IdentityService<CookieIdentityPolicy> {
    IdentityService::new(
        CookieIdentityPolicy::new(&private_key)
            .name("my-app-auth")
            .secure(false)
            .same_site(SameSite::Lax),
    )
}

pub fn app_logger() -> Logger {
    middleware::Logger::default()
}

pub async fn run(addr: String, app_data: AppData) -> std::io::Result<()> {
    info!("Starting server at {}", addr);
    let private_key = rand::thread_rng().gen::<[u8; 32]>();
    let app = move || {
        App::new()
            .configure(configure_app(app_data.clone()))
            .wrap(id_service(&private_key))
            .wrap(app_logger())
    };
    HttpServer::new(app).bind(addr)?.run().await
}