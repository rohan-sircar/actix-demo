#[macro_use]
extern crate diesel;
#[macro_use]
extern crate derive_new;
// #[macro_use]
// extern crate comp;
// #[macro_use]
// extern crate validator_derive;
extern crate bcrypt;
extern crate custom_error;
extern crate regex;
extern crate validator;

use actix_web::{
    error, get, middleware, post, web, App, Error, HttpResponse, HttpServer,
};

use yarte::Template;

use actix_web_httpauth::middleware::HttpAuthentication;

use actix_http::cookie::SameSite;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use rand::Rng;

use actix_files as fs;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
// use middlewares::csrf;
// use routes;
// use routes::users;
// use utils;

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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    // number: i32,
}

#[get("/{id}/{name}")]
async fn index(info: web::Path<(u32, String)>) -> Result<HttpResponse, Error> {
    let (id, name) = (info.0, info.1.clone());
    let template = models::CardTemplate {
        title: "My Title",
        body: name,
        num: id,
    };
    template
        .call()
        .map(|body| HttpResponse::Ok().content_type("text/html").body(body))
        .map_err(|_| {
            error::ErrorInternalServerError("Error while parsing template")
        })
}

/// This handler uses json extractor
#[post("/extractor")]
async fn extract_my_obj(item: web::Json<MyObj>) -> HttpResponse {
    debug!("model: {:?}", item);
    HttpResponse::Ok().json(item.0) // <- send response
}

pub struct UserServiceImpl;

impl UserServiceImpl {
    pub fn new() -> Self {
        UserServiceImpl {}
    }
}

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
            .service(web::scope("/chat").wrap(basic_auth_middleware.clone()))
            // .service(extract_my_obj)
            // .service(index)
            .service(routes::users::get_user)
            .service(routes::users::add_user)
            .service(routes::users::get_all_users)
            .service(routes::auth::login)
            .service(routes::auth::logout)
            .service(routes::auth::index)
            .service(fs::Files::new("/", "./static"))
    };
    HttpServer::new(app).bind(addr)?.run().await
}
