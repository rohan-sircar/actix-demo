use std::future::{ready, Ready};

use actix_http::Payload;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware::Next,
    web::Data,
    Error, FromRequest, HttpRequest,
};
use awc::body::MessageBody;

use crate::{routes::auth::validate_token, AppData};

pub struct CookieAuth {
    pub token: String,
}

impl FromRequest for CookieAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Extract auth_token cookie
        let cookie = req.cookie("auth_token");
        match cookie {
            Some(cookie) => ready(Ok(CookieAuth {
                token: cookie.value().to_string(),
            })),
            None => ready(Err(ErrorUnauthorized("Missing auth cookie"))),
        }
    }
}

pub async fn cookie_auth(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let app_data = req
        .app_data::<Data<AppData>>()
        .cloned()
        .expect("AppData not initialized");

    // Extract cookie
    let cookie = req.cookie("auth_token");
    let token = match cookie {
        Some(cookie) => Ok(cookie.value().to_string()),
        None => Err(ErrorUnauthorized("Missing auth cookie")),
    }?;

    // Validate token using existing logic
    let credentials_repo = &app_data.credentials_repo;
    let jwt_key = &app_data.jwt_key;

    match validate_token(credentials_repo, jwt_key, token).await {
        Ok(_) => Ok(next.call(req).await?),
        Err(err) => Err(Error::from(err)),
    }
}
