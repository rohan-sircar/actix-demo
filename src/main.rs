#[macro_use]
extern crate diesel;
#[macro_use]
extern crate derive_new;
extern crate bcrypt;
extern crate custom_error;
extern crate regex;
extern crate validator;

use actix_web::{cookie::SameSite, middleware, web, App, HttpServer};

use actix_files as fs;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use rand::Rng;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use listenfd::ListenFd;
use types::DbPool;

mod actions;
mod errors;
mod middlewares;
mod models;
mod routes;
mod schema;
mod services;
mod types;
mod utils;

#[macro_use]
extern crate log;

#[derive(Clone)]
pub struct AppConfig {
    hash_cost: u32,
    pool: DbPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    dotenv::dotenv().ok();

    // let _basic_auth_middleware =
    //     HttpAuthentication::basic(utils::auth::validator);

    // set up database connection pool
    let connspec =
        std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    diesel_migrations::run_pending_migrations(&pool.get().unwrap())
        .expect("Error running migrations");

    let hash_cost = std::env::var("HASH_COST")
        .map_err(|e| e.to_string())
        .and_then(|x| x.parse::<u32>().map_err(|e| e.to_string()))
        .unwrap_or_else(|_| {
            info!("Error parsing hash cost env variable, or it is not set. Using default cost of 8");
            8
        });

    let config: AppConfig = AppConfig {
        pool: pool.clone(),
        hash_cost,
    };

    // let user_controller = UserController {
    //     user_service: &user_service,
    // };

    let addr = std::env::var("BIND_ADDRESS").expect("BIND ADDRESS NOT FOUND");
    info!("Starting server {}", addr);
    let private_key = rand::thread_rng().gen::<[u8; 32]>();
    let app = move || {
        App::new()
            .data(config.clone())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("my-app-auth")
                    .secure(false)
                    .same_site(SameSite::Lax),
            ))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/authzd") // endpoint requiring authentication
                    // .wrap(_basic_auth_middleware.clone())
                    .service(routes::users::get_user)
                    .service(routes::users::get_all_users),
            )
            // .route("/api/users/get", web::get().to(user_controller.get_user.into()))
            .service(web::scope("/api/public")) // public endpoint - not implemented yet
            .service(routes::auth::login)
            .service(routes::auth::logout)
            .service(routes::auth::index)
            .service(routes::users::add_user)
            .service(fs::Files::new("/", "./static"))
    };
    // HttpServer::new(app).bind(addr)?.run().await
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(app);
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind(addr)?
    };

    server.run().await
}
