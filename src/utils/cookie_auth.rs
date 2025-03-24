use std::future::{ready, Ready};

use actix_http::{
    header::{self, HeaderMap, HeaderName},
    Payload,
};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware::Next,
    web::Data,
    Error, FromRequest, HttpRequest,
};
use awc::{body::MessageBody, cookie::Cookie, error::HeaderValue};

use crate::{errors::DomainError, routes::auth::validate_token, AppData};

pub struct CookieAuth {
    pub token: String,
}

impl FromRequest for CookieAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Extract auth_token cookie
        let cookie = req.cookie("X-AUTH-TOKEN");
        match cookie {
            Some(cookie) => ready(Ok(CookieAuth {
                token: cookie.value().to_string(),
            })),
            None => ready(Err(ErrorUnauthorized("Missing auth cookie"))),
        }
    }
}

trait HeaderMapExt {
    fn insert_header(
        &mut self,
        name: &'static str,
        value: &str,
    ) -> Result<&mut Self, DomainError>;
}

impl HeaderMapExt for HeaderMap {
    fn insert_header(
        &mut self,
        name: &'static str,
        value: &str,
    ) -> Result<&mut Self, DomainError> {
        self.insert(
            HeaderName::from_static(name),
            HeaderValue::try_from(value)
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Invalid header value {value}: {err}"
                    ))
                })
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to set header {name}: {err}"
                    ))
                })?,
        );
        Ok(self)
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
    let cookie = req.cookie("X-AUTH-TOKEN");
    let token = match cookie {
        Some(cookie) => Ok(cookie.value().to_string()),
        None => Err(ErrorUnauthorized("Missing auth cookie")),
    }?;

    // Validate token using existing logic
    let credentials_repo = &app_data.credentials_repo;
    let jwt_key = &app_data.jwt_key;

    match validate_token(credentials_repo, jwt_key, token).await {
        Ok(session_info) => {
            let mut res = next.call(req).await?;
            // Add custom headers based on session_info
            // Insert session headers
            res.headers_mut()
                .insert_header(
                    "x-session-id",
                    &session_info.session_id.to_string(),
                )?
                .insert_header(
                    "x-session-device-id",
                    &session_info.device_id.to_string(),
                )?
                .insert_header(
                    "x-session-created-at",
                    &session_info
                        .created_at
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                )?
                .insert_header(
                    "x-session-last-used-at",
                    &session_info
                        .last_used_at
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                )?;

            if let Some(device_name) = &session_info.device_name {
                res.headers_mut()
                    .insert_header("x-session-device-name", device_name)?;
            }

            if let Some(ttl_remaining) = &session_info.ttl_remaining {
                res.headers_mut().insert_header(
                    "x-session-ttl-remaining",
                    &ttl_remaining.to_string(),
                )?;
            }

            Ok(res)
        }
        Err(err) => Err(Error::from(err)),
    }
}

pub fn extract_auth_token(headers: &HeaderMap) -> Result<String, DomainError> {
    // Retrieve all set-cookie header values
    let cookie_headers = headers
        .get_all(header::SET_COOKIE)
        .filter_map(|hv| hv.to_str().ok())
        .collect::<Vec<_>>();

    // Parse the cookies using the cookie crate
    let cookies: Vec<Cookie<'_>> = cookie_headers
        .into_iter()
        .filter_map(|s| Cookie::parse(s.to_string()).ok())
        .collect();

    // Look for the cookie named "X-AUTH-TOKEN"
    let token_cookie = cookies
        .into_iter()
        .find(|cookie| cookie.name() == "X-AUTH-TOKEN")
        .ok_or_else(|| {
            DomainError::new_auth_error(
                "Cookie 'X-AUTH-TOKEN' not found".to_owned(),
            )
        })?;

    Ok(token_cookie.value().to_string())
}
