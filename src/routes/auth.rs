use crate::actions::users::get_user_auth_details;
use crate::errors::DomainError;
use crate::models::roles::RoleEnum;
use crate::models::users::{UserId, UserLogin, Username};
use crate::utils::redis_credentials_repo::RedisCredentialsRepo;
use crate::{utils, AppData};
use actix_http::header::{HeaderName, HeaderValue};
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::{self, Data};
use actix_web::{Error, HttpResponse};
use awc::cookie::{Cookie, SameSite};
use bcrypt::verify;
use jwt_simple::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct VerifiedAuthDetails {
    pub user_id: UserId,
    pub username: Username,
    pub roles: Vec<RoleEnum>,
}

#[tracing::instrument(level = "info", skip(req))]
pub async fn extract(
    req: &mut ServiceRequest,
) -> Result<HashSet<RoleEnum>, Error> {
    let app_data = req.app_data::<Data<AppData>>().cloned().unwrap();

    // Extract token from cookie
    let cookie = req
        .cookie("X-AUTH-TOKEN")
        .ok_or_else(|| ErrorUnauthorized("Missing auth cookie"))?;
    let token = cookie.value();

    let claims = utils::get_claims(&app_data.jwt_key, token)?;
    let roles: HashSet<RoleEnum> = claims.custom.roles.into_iter().collect();

    let user_id = claims.custom.user_id.to_string();
    req.headers_mut().insert(
        HeaderName::from_static("x-auth-user"),
        HeaderValue::from_str(&user_id).unwrap(),
    );

    Ok(roles)
}

pub async fn validate_token(
    credentials_repo: &RedisCredentialsRepo,
    jwt_key: &HS256Key,
    token: String,
) -> Result<(), DomainError> {
    let claims = utils::get_claims(jwt_key, &token)?;
    let user_id = claims.custom.user_id;

    let session_token =
        credentials_repo.load(&user_id).await?.ok_or_else(|| {
            DomainError::new_auth_error(format!(
                "Session does not exist for user id - {}",
                &user_id
            ))
        })?;

    if token.eq(&session_token) {
        // Refresh TTL on valid token
        let ttl_seconds: u64 = 1800; // 30 minutes
        credentials_repo.save(&user_id, &token, ttl_seconds).await?;
        Ok(())
    } else {
        Err(DomainError::new_auth_error("Invalid token".to_owned()))
    }
}

#[tracing::instrument(level = "info", skip(app_data))]
pub async fn login(
    user_login: web::Json<UserLogin>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_login = user_login.into_inner().clone();
    let credentials_repo = &app_data.credentials_repo;
    let pool = app_data.pool.clone();
    let mb_user = web::block(move || {
        let mut conn = pool.get()?;
        get_user_auth_details(&user_login.username, &mut conn)
    })
    .await??;
    let user = mb_user.ok_or_else(|| DomainError::AuthError {
        message: "User does not exist".to_owned(),
    })?;
    let valid = web::block(move || {
        verify(user_login.password.as_str(), user.password.as_str())
    })
    .await??;
    let token = if valid {
        let auth_data = VerifiedAuthDetails {
            user_id: user.id,
            username: user.username,
            roles: user.roles,
        };
        let claims =
            Claims::with_custom_claims(auth_data, Duration::from_days(30));
        let token = app_data.jwt_key.authenticate(claims).map_err(|err| {
            DomainError::anyhow_auth("Failed to deserialize token", err)
        })?;

        let ttl_seconds: u64 = 86400; // 24 hours
        let _ = credentials_repo.save(&user.id, &token, ttl_seconds).await?;
        Ok(token)
    } else {
        Err(DomainError::new_auth_error("Wrong password".to_owned()))
    }?;
    let cookie = Cookie::build("X-AUTH-TOKEN", &token)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .finish();
    Ok(HttpResponse::Ok().cookie(cookie).finish())
}
