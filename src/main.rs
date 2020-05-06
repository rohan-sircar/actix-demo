#[macro_use]
extern crate diesel;

use actix_web::{
    dev::ServiceRequest, error, get, middleware, post, web, App, Error, HttpRequest, HttpResponse,
    HttpServer, Responder,
};

use yarte::Template;

use actix_web_httpauth::{extractors::basic::BasicAuth, middleware::HttpAuthentication};

use actix_http::cookie::SameSite;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use rand::Rng;

// use actix_http::*;

use actix_files as fs;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use routes::*;

mod actions;
mod models;
mod routes;
mod schema;
mod types;

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
        .map_err(|_| error::ErrorInternalServerError("Error while parsing template"))
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

pub trait UserService {
    fn do_something(&self);
}

impl UserService for UserServiceImpl {
    fn do_something(&self) {
        println!("hello");
    }
}

fn fun1(user_service: &dyn UserService) {
    user_service.do_something();
}

fn fun2<T>(user_service: T)
where
    T: UserService,
{
    user_service.do_something();
}

/// In this example validator returns immediately,
/// but since it is required to return anything
/// that implements `IntoFuture` trait,
/// it can be extended to query database
/// or to do something else in a async manner.
async fn validator(req: ServiceRequest, credentials: BasicAuth) -> Result<ServiceRequest, Error> {
    // All users are great and more than welcome!
    // let pool = req.app_data::<DbPool>();
    // let maybe_header = req.headers().get("Authorization");
    // match maybe_header {
    //     Some(value) => {
    //         info!("{:?}", *value);
    //         let x: Result<Basic, _> = Scheme::parse(value);
    //         let y = x.expect("Error parsing header");
    //         println!("{}", y.user_id());
    //         println!("{:?}", y.password().clone());
    //     }
    //     None => debug!("Header not found"),
    // }

    // maybe_header
    //     .map(|value| {
    //         let x: Result<Basic, _> = Scheme::parse(value);
    //         x
    //     })
    //     .map(|maybe_basic| {
    //         maybe_basic
    //             .map(|x| {
    //                 println!("{}", x.user_id());
    //                 println!("{:?}", x.password().clone());
    //             })
    //             .map_err(|x| println!("error parsing reason - {}", x.to_string()))
    //         // maybe_basic
    //     });
    // let auth = Authorization::<Basic>;
    println!("{}", credentials.user_id());
    println!("{:?}", credentials.password());
    Ok(req)
}

// fn parse(header: &HeaderValue) -> Result<Basic, ParseError> {
//     // "Basic *" length
//     if header.len() < 7 {
//         return Err(ParseError::Invalid);
//     }

//     let mut parts = header.to_str()?.splitn(2, ' ');
//     match parts.next() {
//         Some(scheme) if scheme == "Basic" => (),
//         _ => return Err(ParseError::MissingScheme),
//     }

//     let decoded = base64::decode(parts.next().ok_or(ParseError::Invalid)?)?;
//     let mut credentials = str::from_utf8(&decoded)?.splitn(2, ':');

//     let user_id = credentials
//         .next()
//         .ok_or(ParseError::MissingField("user_id"))
//         .map(|user_id| user_id.to_string().into())?;
//     let password = credentials
//         .next()
//         .ok_or(ParseError::MissingField("password"))
//         .map(|password| {
//             if password.is_empty() {
//                 None
//             } else {
//                 Some(password.to_string().into())
//             }
//         })?;

//     Ok(Basic { user_id, password })
// }

#[get("/login")]
async fn login(id: Identity) -> HttpResponse {
    let maybe_identity = id.identity();
    // id.remember("user1".to_owned());
    let response = if let Some(identity) = maybe_identity {
        HttpResponse::Ok()
            .header("location", "/")
            .content_type("text/plain")
            .body(format!("Already logged in {}", identity))
    } else {
        id.remember("user1".to_owned());
        HttpResponse::Found().header("location", "/").finish()
    };
    // HttpResponse::Found().header("location", "/").finish()
    response
}

#[get("/logout")]
async fn logout(id: Identity) -> HttpResponse {
    let maybe_identity = id.identity();
    // id.remember("user1".to_owned());
    let response = if let Some(identity) = maybe_identity {
        info!("Logging out {user}", user = identity);
        id.forget();
        HttpResponse::Found().header("location", "/").finish()
    } else {
        HttpResponse::Ok()
            .header("location", "/")
            .content_type("text/plain")
            .body("Not logged in")
    };
    // id.forget();
    // HttpResponse::Found().header("location", "/").finish()
    response
}

#[get("/")]
async fn index2(id: Identity) -> String {
    format!(
        "Hello {}",
        id.identity().unwrap_or_else(|| "Anonymous".to_owned())
    )
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    dotenv::dotenv().ok();

    let user_service: Box<dyn UserService> = Box::new(UserServiceImpl::new());
    user_service.do_something();

    fun1(user_service.as_ref());

    let user_service_impl = UserServiceImpl::new();
    fun2(user_service_impl);

    let basic_auth_middleware = HttpAuthentication::basic(validator);

    // fun1(Rc::clone(&user_service).as_ref());
    // set up database connection pool
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

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
                    .same_site(SameSite::Lax), // .same_site(),
            ))
            .wrap(middleware::Logger::default())
            .service(web::scope("/chat").wrap(basic_auth_middleware.clone()))
            // .service(extract_my_obj)
            // .service(index)
            .service(get_user)
            .service(add_user)
            .service(get_all_users)
            .service(login)
            .service(logout)
            .service(index2)
            .service(fs::Files::new("/", "./static"))
    };
    HttpServer::new(app).bind(addr)?.run().await
}
