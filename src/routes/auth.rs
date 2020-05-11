use crate::types::DbPool;
use actix_threadpool::BlockingError;
use actix_web::{web, ResponseError};
use actix_web_httpauth::extractors::basic::BasicAuth;

use crate::actions::users;
use crate::errors;
use actix_identity::Identity;
use actix_web::{get, Error, HttpResponse};

#[get("/login")]
pub async fn login(
    id: Identity,
    credentials: BasicAuth,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, impl ResponseError> {
    let maybe_identity = id.identity();
    let response = if let Some(identity) = maybe_identity {
        Ok(HttpResponse::Found()
            .header("location", "/")
            .content_type("text/plain")
            .json(format!("Already logged in as {}", identity)))
    } else {
        let credentials2 = credentials.clone();
        web::block(move || validate_basic_auth(credentials2, &pool))
            .await
            .and_then(|valid| {
                if valid {
                    id.remember(credentials.user_id().to_string());
                    Ok(HttpResponse::Found().header("location", "/").finish())
                } else {
                    Err(BlockingError::Error(
                        errors::DomainError::new_password_error(
                            "Wrong password or account does not exist"
                                .to_string(),
                        ),
                    ))
                }
            })
    };
    // println!("{}", credentials.user_id());
    // println!("{:?}", credentials.password());
    response
}

#[get("/logout")]
pub async fn logout(
    id: Identity,
    _credentials: BasicAuth,
) -> Result<HttpResponse, Error> {
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

fn validate_basic_auth(
    credentials: BasicAuth,
    pool: &web::Data<DbPool>,
) -> Result<bool, errors::DomainError> {
    let result = if let Some(password_ref) = credentials.password() {
        let conn = pool.get()?;
        let password = password_ref.clone().into_owned();
        let valid = users::verify_password(
            credentials.user_id().clone().into_owned(),
            password,
            &conn,
        )?;
        Ok(valid)
    } else {
        Err(errors::DomainError::new_password_error(
            "No password given".to_owned(),
        ))
    };
    result
}
