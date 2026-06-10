use crate::actions::users::get_user_auth_details;
use crate::errors::DomainError;
use crate::models::roles::RoleEnum;
use crate::models::session::{SessionInfo, SessionStatus};
use crate::models::users::{Email, UserLogin, UserUuid, Username};
use crate::services::email::tokens;
use crate::utils::redis_credentials_repo::RedisCredentialsRepo;
use crate::{diesel, utils, AppData};
use actix_http::header::{HeaderName, HeaderValue};
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::{self, Data};
use actix_web::{Error, HttpRequest, HttpResponse};
use awc::cookie::{Cookie, SameSite};
use bcrypt::{hash, verify};
use chrono::Utc;
use diesel::Connection;
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct VerifiedAuthDetails {
    pub user_uuid: UserUuid,
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

    let user_uuid = claims.custom.user_uuid.to_string();
    req.headers_mut().insert(
        HeaderName::from_static("x-auth-user"),
        HeaderValue::from_str(&user_uuid).unwrap(),
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
    let user_uuid = claims.custom.user_uuid;
    let session_id = claims.custom.session_id;

    // Clean up expired tokens first
    // let _ = credentials_repo.cleanup_expired_tokens(&user_uuid).await?;

    // Check if this specific token exists in the user's sessions
    let mb_session_info = credentials_repo
        .load_session(&user_uuid, &session_id)
        .await?;

    let _ = tracing::debug!("Retrieved session info {mb_session_info:?}");

    match mb_session_info {
        Some(session_info) => {
            // Check if the expiry key exists
            let status = credentials_repo
                .is_token_expired(&user_uuid, &session_id)
                .await?;
            if status == SessionStatus::Expired {
                // Token has expired
                let _ = credentials_repo
                    .delete_session(&user_uuid, &session_id)
                    .await?;
                return Err(DomainError::new_auth_error(
                    "Token has expired".to_owned(),
                ));
            }

            // Update last used time and refresh TTL
            let session_info = credentials_repo
                .update_session_last_used(&session_id, session_info, &user_uuid)
                .await?;
            Ok(session_info)
        }
        None => Err(DomainError::new_auth_error(format!(
            "Session does not exist for user - {}",
            &user_uuid
        ))),
    }
}

#[utoipa::path(
    post,
    path = "/api/login",
    tag = "auth",
    request_body = UserLogin,
    responses(
        (status = 200, description = "Login successful - sets auth cookie"),
        (status = 401, description = "Invalid credentials", body = ErrorResponseString),
    ),
)]
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
        user_uuid: user.user_uuid,
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
    let now = Utc::now().naive_utc();

    let ttl_seconds = app_data.config.session.expiration_secs;
    let session_info = SessionInfo {
        session_id,
        device_id,
        device_name: login_request.device_name,
        created_at: now,
        last_used_at: now,
        token: token.clone(),
        ttl_remaining: Some(ttl_seconds as i64),
    };

    // create session
    let _ = credentials_repo
        .create_session(
            &user.user_uuid,
            &session_id,
            &session_info,
            ttl_seconds,
        )
        .await?;

    let cookie = Cookie::build("X-AUTH-TOKEN", &token)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).finish())
}

#[utoipa::path(
    get,
    path = "/api/sessions",
    tag = "auth",
    responses(
        (status = 200, description = "List of active sessions", body = Vec<SessionInfo>),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
// New endpoint to list all active sessions for a user
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn list_sessions(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;

    let credentials_repo = &app_data.credentials_repo;

    let sessions = credentials_repo.load_all_sessions(&user_uuid).await?;

    Ok(HttpResponse::Ok().json(sessions))
}

#[utoipa::path(
    post,
    path = "/api/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logout successful - clears auth cookie"),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
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
    let user_uuid = claims.custom.user_uuid;
    let session_id = claims.custom.session_id;
    // Check if the session exists
    let _session = credentials_repo
        .load_session(&user_uuid, &session_id)
        .await?
        .ok_or_else(|| {
            DomainError::new_auth_error("Session not found".to_owned())
        })?;
    // Delete the session
    let _ = credentials_repo
        .delete_session(&user_uuid, &session_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    path = "/api/sessions/{session_id}",
    tag = "auth",
    params(
        ("session_id" = String, Path, description = "Session ID to revoke"),
    ),
    responses(
        (status = 200, description = "Session revoked successfully"),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
        (status = 404, description = "Session not found", body = ErrorResponseString),
    ),
)]
// New endpoint to revoke a specific session
#[tracing::instrument(level = "info", skip(app_data, session_id, req))]
pub async fn revoke_session(
    req: HttpRequest,
    session_id: web::Path<String>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;

    let credentials_repo = &app_data.credentials_repo;

    let session_id = session_id.into_inner();
    let session_id = Uuid::parse_str(&session_id).map_err(|err| {
        DomainError::new_bad_input_error(format!(
            "Invalid session id: {session_id} err: {err}"
        ))
    })?;

    // Check if the session exists
    let session = credentials_repo
        .load_session(&user_uuid, &session_id)
        .await?;
    if session.is_none() {
        return Err(DomainError::new_auth_error(
            "Session not found".to_owned(),
        ));
    }

    // Delete the session
    let _ = credentials_repo
        .delete_session(&user_uuid, &session_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    post,
    path = "/api/sessions/revoke-others",
    tag = "auth",
    responses(
        (status = 200, description = "All other sessions revoked successfully"),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
// New endpoint to revoke all sessions except the current one
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn revoke_other_sessions(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;
    // Extract token from cookie
    let cookie = req.cookie("X-AUTH-TOKEN").ok_or_else(|| {
        DomainError::new_auth_error("Missing auth token".to_owned())
    })?;
    let current_token = cookie.value();
    let credentials_repo = &app_data.credentials_repo;
    let jwt_key = &app_data.jwt_key;
    let claims = utils::get_claims(jwt_key, current_token)?;
    let current_session_id = claims.custom.session_id;

    // Get all sessions
    let sessions = credentials_repo.load_all_sessions(&user_uuid).await?;

    // Delete all sessions except the current one
    for (session_id, _) in sessions {
        if session_id != current_session_id {
            credentials_repo
                .delete_session(&user_uuid, &session_id)
                .await?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordResetRequest {
    pub email: Email,
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordResetCompleteRequest {
    pub token: String,
    pub new_password: String,
}

#[utoipa::path(
    post,
    path = "/api/email/verify",
    tag = "auth",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verification result"),
    ),
)]
#[tracing::instrument(level = "info", skip(app_data, form))]
pub async fn verify_email(
    app_data: web::Data<AppData>,
    form: web::Json<VerifyEmailRequest>,
) -> Result<HttpResponse, DomainError> {
    let token = form.into_inner().token;
    let thash = tokens::hash_token(&token);

    let result = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;

        use crate::schema::email_verification_tokens::dsl::*;

        let result = conn.transaction::<_, DomainError, _>(|conn| {
            let token_record = email_verification_tokens
                .select((user_id, expires_at))
                .filter(token_hash.eq(&thash))
                .filter(used.eq(false))
                .for_update()
                .first::<(i32, chrono::NaiveDateTime)>(conn)
                .optional()
                .map_err(|e| {
                    DomainError::new_internal_error(format!(
                        "Database error: {e}"
                    ))
                })?;

            match token_record {
                None => Ok(None),
                Some((uid, token_expires_at)) => {
                    if chrono::Utc::now().naive_utc() > token_expires_at {
                        return Err(DomainError::new_field_validation_error(
                            "Verification link has expired".to_string(),
                        ));
                    }

                    diesel::update(email_verification_tokens)
                        .filter(token_hash.eq(&thash))
                        .set(used.eq(true))
                        .execute(conn)?;

                    Ok(Some(uid))
                }
            }
        });

        result
    })
    .await??;

    if let Some(uid) = result {
        tracing::info!(user_id = %uid, "Email verified successfully");
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Email verified successfully"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "If the email is registered, you will receive a verification email"
    })))
}

#[utoipa::path(
    post,
    path = "/api/password-reset/request",
    tag = "auth",
    request_body = PasswordResetRequest,
    responses(
        (status = 200, description = "Password reset email sent if email is registered"),
    ),
)]
#[tracing::instrument(level = "info", skip(app_data, form))]
pub async fn request_password_reset(
    app_data: web::Data<AppData>,
    form: web::Json<PasswordResetRequest>,
) -> Result<HttpResponse, DomainError> {
    let email = form.into_inner().email;
    let mailer = app_data.mailer.clone();
    let email_clone = email.clone();
    let pool_clone = app_data.pool.clone();
    let ttl_secs = app_data.config.email_token_ttl_reset_secs;

    let result = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::users::find_user_by_email(&email_clone, &mut conn)
    })
    .await??;

    if let Some(user) = result {
        let uid: i32 = user.id.as_uint() as i32;
        let user_name = user.username.as_str().to_string();

        let token = tokens::generate_token();
        let thash = tokens::hash_token(&token);

        tokio::spawn(async move {
            if let Ok(mut conn) = pool_clone.get() {
                use crate::schema::password_reset_tokens::dsl::*;

                if let Err(e) = diesel::insert_into(password_reset_tokens)
                    .values((
                        user_id.eq(uid),
                        token_hash.eq(&thash),
                        expires_at.eq(chrono::Utc::now().naive_utc()
                            + chrono::Duration::seconds(ttl_secs as i64)),
                    ))
                    .execute(&mut conn)
                {
                    tracing::error!(error = %e, uid = %uid, "Failed to store password reset token");
                }
            }

            if let Err(e) = mailer
                .send_reset_email(email.as_str(), &user_name, &token)
                .await
            {
                tracing::error!(error = %e, "Failed to send password reset email");
            }
        });
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "If the email is registered, you will receive a password reset link"
    })))
}

#[utoipa::path(
    post,
    path = "/api/password-reset/complete",
    tag = "auth",
    request_body = PasswordResetCompleteRequest,
    responses(
        (status = 200, description = "Password reset result"),
    ),
)]
#[tracing::instrument(level = "info", skip(app_data, form))]
pub async fn complete_password_reset(
    app_data: web::Data<AppData>,
    form: web::Json<PasswordResetCompleteRequest>,
) -> Result<HttpResponse, DomainError> {
    let req = form.into_inner();
    let token = req.token;
    let new_password = req.new_password;
    let thash = tokens::hash_token(&token);
    let hash_cost = app_data.config.hash_cost;

    let result = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;

        use crate::schema::password_reset_tokens::dsl::*;
        use crate::schema::users::dsl as users;

        let result = conn.transaction::<_, DomainError, _>(|conn| {
            let token_record = password_reset_tokens
                .select((user_id, expires_at))
                .filter(token_hash.eq(&thash))
                .filter(used.eq(false))
                .for_update()
                .first::<(i32, chrono::NaiveDateTime)>(conn)
                .optional()
                .map_err(|e| {
                    DomainError::new_internal_error(format!(
                        "Database error: {e}"
                    ))
                })?;

            match token_record {
                None => Ok(None),
                Some((uid, token_expires_at)) => {
                    if chrono::Utc::now().naive_utc() > token_expires_at {
                        return Err(DomainError::new_field_validation_error(
                            "Reset link has expired".to_string(),
                        ));
                    }

                    diesel::update(password_reset_tokens)
                        .filter(token_hash.eq(&thash))
                        .set(used.eq(true))
                        .execute(conn)?;

                    let hashed_password = hash(new_password, hash_cost)?;

                    diesel::update(users::users.filter(users::id.eq(uid)))
                        .set(users::password.eq(hashed_password))
                        .execute(conn)?;

                    Ok(Some(uid))
                }
            }
        });

        result
    })
    .await??;

    if let Some(uid) = result {
        tracing::info!(user_id = %uid, "Password reset successfully");
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Password reset successfully"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "If the token is valid, the password will be reset"
    })))
}
