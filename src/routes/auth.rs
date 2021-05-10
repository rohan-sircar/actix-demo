use actix_web::web;
use actix_web_httpauth::extractors::basic::BasicAuth;

use crate::errors::DomainError;
use crate::{actions::users, AppData};
use actix_identity::Identity;
use actix_web::{get, Error, HttpResponse};

#[get("/login")]
pub async fn login(
    id: Identity,
    credentials: BasicAuth,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let maybe_identity = id.identity();
    let response = if let Some(identity) = maybe_identity {
        Ok(HttpResponse::Found()
            .header("location", "/")
            .content_type("text/plain")
            .json(format!("Already logged in as {}", identity)))
    } else {
        let credentials2 = credentials.clone();
        let valid =
            web::block(move || validate_basic_auth(credentials2, &app_data))
                .await
                .map_err(|_err| {
                    DomainError::new_thread_pool_error(_err.to_string())
                })?;
        if valid {
            id.remember(credentials.user_id().to_string());
            Ok(HttpResponse::Found().header("location", "/").finish())
        } else {
            Err(DomainError::new_auth_error(
                "Wrong password or account does not exist".to_owned(),
            ))
        }
    };
    response
}

//TODO: fix the response
#[get("/logout")]
pub async fn logout(
    id: Identity,
    _credentials: BasicAuth,
) -> Result<HttpResponse, Error> {
    let maybe_identity = id.identity();
    let response = if let Some(identity) = maybe_identity {
        tracing::info!("Logging out {user}", user = identity);
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

/// basic auth middleware function
pub fn validate_basic_auth(
    credentials: BasicAuth,
    app_data: &AppData,
) -> Result<bool, DomainError> {
    let result = if let Some(password_ref) = credentials.password() {
        let pool = &app_data.pool;
        let conn = pool.get()?;
        let password = password_ref.clone().into_owned();
        let valid = users::verify_password(
            &credentials.user_id().clone().into_owned(),
            &password,
            &conn,
        )?;
        Ok(valid)
    } else {
        Err(DomainError::new_password_error(
            "No password given".to_owned(),
        ))
    };
    result
}
