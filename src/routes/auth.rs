use actix_web_httpauth::extractors::basic::BasicAuth;

use actix_identity::Identity;
use actix_web::{get, post, Error, HttpResponse};

#[get("/login")]
pub async fn login(id: Identity, credentials: BasicAuth) -> Result<HttpResponse, Error> {
    let maybe_identity = id.identity();
    let response = if let Some(identity) = maybe_identity {
        HttpResponse::Found()
            .header("location", "/")
            .content_type("text/plain")
            .json(format!("Already logged in as {}", identity))
    } else {
        id.remember(credentials.user_id().to_string());
        HttpResponse::Found().header("location", "/").finish()
    };
    println!("{}", credentials.user_id());
    println!("{:?}", credentials.password());
    Ok(response)
}

#[get("/logout")]
pub async fn logout(id: Identity, _credentials: BasicAuth) -> Result<HttpResponse, Error> {
    let maybe_identity = id.identity();
    let response = if let Some(identity) = maybe_identity {
        info!("Logging out {user}", user = identity);
        id.forget();
        HttpResponse::Found().header("location", "/").finish()
    } else {
        HttpResponse::Found()
            .header("location", "/")
            .content_type("text/plain")
            .json("Not logged in")
    };
    Ok(response)
}

#[get("/")]
pub async fn index(id: Identity) -> String {
    format!(
        "Hello {}",
        id.identity().unwrap_or_else(|| "Anonymous".to_owned())
    )
}
