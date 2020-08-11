#[macro_use]
extern crate diesel;
#[macro_use]
extern crate derive_new;
extern crate bcrypt;
extern crate custom_error;
extern crate regex;
extern crate validator;

use actix_web::{
    middleware, web, App, HttpServer,
};


use actix_web_httpauth::middleware::HttpAuthentication;

use actix_http::cookie::SameSite;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use rand::Rng;

use actix_files as fs;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod actions;
mod errors;
mod middlewares;
mod models;
mod routes;
mod schema;
mod types;
mod utils;

#[macro_use]
extern crate log;


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    dotenv::dotenv().ok();

    let basic_auth_middleware =
        HttpAuthentication::basic(utils::auth::validator);

    // set up database connection pool
    let connspec =
        std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    diesel_migrations::run_pending_migrations(&pool.get().unwrap())
        .expect("Error running migrations");

    let addr = std::env::var("BIND_ADDRESS").expect("BIND ADDRESS NOT FOUND");
    info!("Starting server {}", addr);
    let private_key = rand::thread_rng().gen::<[u8; 32]>();
    let app = move || {
        App::new()
            .data(pool.clone())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("my-app-auth")
                    .secure(false)
                    .same_site(SameSite::Lax),
            ))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/authzd") // endpoint requiring authentication
                    .wrap(basic_auth_middleware.clone())
                    .service(routes::users::get_user)
                    .service(routes::users::get_all_users),
            )
            .service(web::scope("/api/public")) // public endpoint - not implemented yet
            .service(routes::auth::login)
            .service(routes::auth::logout)
            .service(routes::auth::index)
            .service(routes::users::add_user)
            .service(fs::Files::new("/", "./static"))
    };
    HttpServer::new(app).bind(addr)?.run().await
}
