use crate::actions::get_user_auth_details;
use crate::errors::DomainError;
use crate::models::roles::RoleEnum;
use crate::models::{UserId, UserLogin, Username};
use crate::AppData;
use actix_http::Payload;
use actix_web::dev::ServiceRequest;
use actix_web::web::{self, Data};
use actix_web::{post, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use bcrypt::verify;
use jwt_simple::prelude::*;

#[derive(Serialize, Deserialize)]
struct VerifiedAuthDetails {
    user_id: UserId,
    username: Username,
    roles: Vec<RoleEnum>,
}

pub async fn extract(req: &mut ServiceRequest) -> Result<Vec<RoleEnum>, Error> {
    let app_data = req.app_data::<Data<AppData>>().cloned().unwrap();
    let bearer = req.extract::<BearerAuth>().await?;
    let claims = app_data
        .jwt_key
        .verify_token::<VerifiedAuthDetails>(bearer.token(), None)
        .map_err(|err| {
            Error::from(DomainError::anyhow_auth("Failed to verify token", err))
        })?;
    let roles = claims.custom.roles;

    Ok(roles)
}

#[tracing::instrument(level = "info", skip(req))]
pub async fn validate_bearer_auth(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let app_data = req.app_data::<Data<AppData>>().cloned().unwrap();
    let token: String = credentials.token().into();
    let (http_req, payload) = req.into_parts();
    let claims = app_data
        .jwt_key
        .verify_token::<VerifiedAuthDetails>(&token, None)
        .map_err(|err| {
            (
                Error::from(DomainError::anyhow_auth(
                    "Failed to verify token",
                    err,
                )),
                ServiceRequest::from_parts(http_req.clone(), Payload::None),
            )
        })?;
    let user_id = claims.custom.user_id;
    let credentials_repo = &app_data.credentials_repo;

    let session_token =
        match credentials_repo.load(&user_id).await.map_err(|err| {
            (
                Error::from(err),
                ServiceRequest::from_parts(http_req.clone(), Payload::None),
            )
        })? {
            Some(value) => Ok(value),
            None => Err((
                Error::from(DomainError::new_auth_error(format!(
                    "Session does not exist for user id - {}",
                    &user_id
                ))),
                ServiceRequest::from_parts(http_req.clone(), Payload::None),
            )),
        }?;

    if token.eq(&session_token) {
        Ok(ServiceRequest::from_parts(http_req, payload))
    } else {
        Err((
            Error::from(DomainError::new_auth_error(
                "Invalid token".to_owned(),
            )),
            ServiceRequest::from_parts(http_req.clone(), Payload::None),
        ))
    }
}

#[tracing::instrument(level = "info", skip(app_data))]
#[post("/api/login")]
pub async fn login(
    user_login: web::Json<UserLogin>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_login = user_login.into_inner().clone();
    let credentials_repo = &app_data.credentials_repo;
    let pool = app_data.pool.clone();
    let mb_user = web::block(move || {
        let conn = pool.get()?;
        get_user_auth_details(&user_login.username, &conn)
    })
    .await??;
    let token = match mb_user {
        Some(user) => {
            let valid = web::block(move || {
                verify(user_login.password.as_str(), user.password.as_str())
            })
            .await??;
            if valid {
                let auth_data = VerifiedAuthDetails {
                    user_id: user.id.clone(),
                    username: user.username,
                    roles: user.roles,
                };
                let claims = Claims::with_custom_claims(
                    auth_data,
                    Duration::from_hours(2),
                );
                let token =
                    app_data.jwt_key.authenticate(claims).map_err(|err| {
                        DomainError::anyhow_auth(
                            "Failed to deserialize token",
                            err,
                        )
                    })?;

                let _ = credentials_repo.save(&user.id, &token).await?;
                Ok(token)
            } else {
                Err(DomainError::new_auth_error("Wrong password".to_owned()))
            }
        }
        None => Err(DomainError::AuthError {
            message: "User does not exist".to_owned(),
        }),
    }?;

    Ok(HttpResponse::Ok()
        .insert_header(("X-AUTH-TOKEN", token))
        .finish())
}
