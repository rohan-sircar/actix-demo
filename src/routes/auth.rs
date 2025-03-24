use crate::actions::users::get_user_auth_details;
use crate::errors::DomainError;
use crate::models::roles::RoleEnum;
use crate::models::session::{SessionInfo, SessionStatus};
use crate::models::users::{UserId, UserLogin, Username};
use crate::utils::redis_credentials_repo::RedisCredentialsRepo;
use crate::{utils, AppData};
use actix_http::header::{HeaderName, HeaderValue};
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::{self, Data};
use actix_web::{Error, HttpRequest, HttpResponse};
use awc::cookie::{Cookie, SameSite};
use bcrypt::verify;
use jwt_simple::prelude::*;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct VerifiedAuthDetails {
    pub user_id: UserId,
    pub session_id: Uuid,
    pub username: Username,
    pub roles: Vec<RoleEnum>,
    pub device_id: String,
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

    // Also add device ID to headers
    req.headers_mut().insert(
        HeaderName::from_static("x-auth-device"),
        HeaderValue::from_str(&claims.custom.device_id).map_err(|err| {
            ErrorUnauthorized(format!("Invalid device ID: {err}"))
        })?,
    );

    Ok(roles)
}

pub async fn validate_token(
    credentials_repo: &RedisCredentialsRepo,
    jwt_key: &HS256Key,
    token: String,
) -> Result<SessionInfo, DomainError> {
    let claims = utils::get_claims(jwt_key, &token)?;
    let user_id = claims.custom.user_id;
    let session_id = claims.custom.session_id;

    // Clean up expired tokens first
    // let _ = credentials_repo.cleanup_expired_tokens(&user_id).await?;

    // Check if this specific token exists in the user's sessions
    let mb_session_info =
        credentials_repo.load_session(&user_id, &session_id).await?;

    let _ = tracing::debug!("Retrieved session info {mb_session_info:?}");

    match mb_session_info {
        Some(session_info) => {
            // Check if the expiry key exists
            let status = credentials_repo
                .is_token_expired(&user_id, &session_id)
                .await?;
            if status == SessionStatus::Expired {
                // Token has expired
                let _ = credentials_repo
                    .delete_session(&user_id, &session_id)
                    .await?;
                return Err(DomainError::new_auth_error(
                    "Token has expired".to_owned(),
                ));
            }

            // Update last used time and refresh TTL
            let session_info = credentials_repo
                .update_session_last_used(session_info, &user_id)
                .await?;
            Ok(session_info)
        }
        None => Err(DomainError::new_auth_error(format!(
            "Session does not exist for user id - {}",
            &user_id
        ))),
    }
}

#[tracing::instrument(level = "info", skip(app_data, login_request))]
pub async fn login(
    login_request: web::Json<UserLogin>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let credentials_repo = &app_data.credentials_repo;
    let pool = app_data.pool.clone();

    let login_request = login_request.into_inner();

    let mb_user = web::block(move || {
        let mut conn = pool.get()?;
        get_user_auth_details(&login_request.username, &mut conn)
    })
    .await??;

    let user = mb_user.ok_or_else(|| DomainError::AuthError {
        message: "User does not exist".to_owned(),
    })?;

    let valid = web::block(move || {
        verify(login_request.password.as_str(), user.password.as_str())
    })
    .await??;

    if !valid {
        return Err(DomainError::new_auth_error("Wrong password".to_owned()));
    };

    let session_id = Uuid::new_v4();
    // Generate a unique device ID if not provided
    let device_id = Uuid::new_v4();

    let auth_data = VerifiedAuthDetails {
        user_id: user.id,
        session_id,
        username: user.username,
        roles: user.roles,
        device_id: device_id.to_string(),
    };

    let claims = Claims::with_custom_claims(auth_data, Duration::from_days(30));
    let token = app_data.jwt_key.authenticate(claims).map_err(|err| {
        DomainError::anyhow_auth("Failed to deserialize token", err)
    })?;

    // Create session info
    let now = chrono::Utc::now().naive_utc();

    let ttl_seconds = app_data.config.session.expiration_secs;
    let session_info = SessionInfo {
        session_id,
        device_id,
        device_name: login_request.device_name.clone(),
        created_at: now,
        last_used_at: now,
        token: token.clone(),
        ttl_remaining: Some(ttl_seconds as i64),
    };

    // create session
    let _ = credentials_repo
        .create_session(&user.id, &session_id, &session_info, ttl_seconds)
        .await?;

    let cookie = Cookie::build("X-AUTH-TOKEN", &token)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).finish())
}

// New endpoint to list all active sessions for a user
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn list_sessions(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_id = utils::extract_user_id_from_header(req.headers())?;

    let credentials_repo = &app_data.credentials_repo;

    let sessions = credentials_repo.load_all_sessions(&user_id).await?;

    // let sessions: Vec<_> = sessions.into_values().collect();

    Ok(HttpResponse::Ok().json(sessions))
}

// New endpoint to revoke a specific session
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn logout(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    // Extract token from cookie
    let cookie = req.cookie("X-AUTH-TOKEN").ok_or_else(|| {
        DomainError::new_auth_error("Missing auth token".to_owned())
    })?;
    let token = cookie.value();
    let credentials_repo = &app_data.credentials_repo;
    let jwt_key = &app_data.jwt_key;
    let claims = utils::get_claims(jwt_key, token)?;
    let user_id = claims.custom.user_id;
    let session_id = claims.custom.session_id;
    // Check if the session exists
    let _session = credentials_repo
        .load_session(&user_id, &session_id)
        .await?
        .ok_or_else(|| {
            DomainError::new_auth_error("Session not found".to_owned())
        })?;
    // Delete the session
    let _ = credentials_repo
        .delete_session(&user_id, &session_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

// New endpoint to revoke a specific session
#[tracing::instrument(level = "info", skip(app_data, session_id, req))]
pub async fn revoke_session(
    req: HttpRequest,
    session_id: web::Path<String>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_id = utils::extract_user_id_from_header(req.headers())?;

    let credentials_repo = &app_data.credentials_repo;

    let session_id = session_id.into_inner();
    let session_id = Uuid::parse_str(&session_id).unwrap();

    // Check if the session exists
    let session = credentials_repo.load_session(&user_id, &session_id).await?;
    if session.is_none() {
        return Err(DomainError::new_auth_error(
            "Session not found".to_owned(),
        ));
    }

    // Delete the session
    let _ = credentials_repo
        .delete_session(&user_id, &session_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

// // New endpoint to revoke all sessions except the current one
// #[tracing::instrument(level = "info", skip(app_data, req))]
// pub async fn revoke_other_sessions(
//     req: HttpRequest,
//     app_data: web::Data<AppData>,
// ) -> Result<HttpResponse, DomainError> {
//     let user_id = utils::extract_user_id_from_header(req.headers())?;

//     let credentials_repo = &app_data.credentials_repo;

//     // Get the current token from cookie
//     let current_token = req
//         .cookie("X-AUTH-TOKEN")
//         .map(|c| c.value().to_string())
//         .ok_or_else(|| {
//             DomainError::new_auth_error("Missing auth cookie".to_owned())
//         })?;

//     // Get all sessions
//     let sessions = credentials_repo.load_all_sessions(&user_id).await?;

//     // Delete all sessions except the current one
//     for (token, _) in sessions {
//         if token != current_token {
//             credentials_repo.delete_session(&user_id, &token).await?;
//         }
//     }

//     Ok(HttpResponse::Ok().finish())
// }
