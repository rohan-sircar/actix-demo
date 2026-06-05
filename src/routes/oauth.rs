use crate::actions::users::find_or_create_oauth_user;
use crate::errors::DomainError;
use crate::models::session::SessionInfo;
use crate::models::users::OAuthProvider;
use crate::routes::auth::VerifiedAuthDetails;
use crate::services::oauth;
use crate::AppData;
use actix_web::web::{self, Data};
use actix_web::HttpResponse;
use awc::cookie::{Cookie, SameSite};
use chrono::Utc;
use jwt_simple::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct OAuthCallbackQuery {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/auth/oauth/github/login",
    tag = "oauth",
    responses(
        (status = 307, description = "Redirects to GitHub OAuth"),
        (status = 401, description = "OAuth is not enabled", body = crate::models::misc::ErrorResponse<String>),
    ),
)]
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn github_login(
    app_data: Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    if !app_data.config.oauth.enabled {
        return Err(DomainError::new_auth_error(
            "OAuth is not enabled".to_owned(),
        ));
    }

    let redis = app_data.redis_conn_manager.clone();
    let prefix = &app_data.redis_prefix;

    let (state, code_challenge) =
        oauth::generate_state_and_challenge(redis, prefix).await?;

    let authorize_url = oauth::build_github_authorize_url(
        &app_data.config.oauth,
        &state,
        &code_challenge,
    )?;

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", authorize_url))
        .finish())
}

#[utoipa::path(
    get,
    path = "/api/auth/oauth/github/callback",
    tag = "oauth",
    responses(
        (status = 307, description = "Redirects to app root after successful login"),
        (status = 400, description = "Invalid OAuth callback parameters", body = crate::models::misc::ErrorResponse<String>),
    ),
)]
#[tracing::instrument(level = "info", skip(app_data, query))]
pub async fn github_callback(
    app_data: Data<AppData>,
    query: web::Query<OAuthCallbackQuery>,
) -> Result<HttpResponse, DomainError> {
    if let Some(ref error) = query.error {
        tracing::warn!(error = %error, "OAuth error from GitHub");
        return Ok(HttpResponse::BadRequest()
            .content_type("application/json")
            .body(format!("{{\"error\":\"{}\"}}", error)));
    }

    let state = query.state.as_deref().ok_or_else(|| {
        DomainError::new_bad_input_error("Missing state parameter".to_owned())
    })?;

    let code = query.code.as_deref().ok_or_else(|| {
        DomainError::new_bad_input_error("Missing code parameter".to_owned())
    })?;

    let config = &app_data.config.oauth;
    if !config.enabled {
        return Err(DomainError::new_auth_error(
            "OAuth is not enabled".to_owned(),
        ));
    }

    let redis = app_data.redis_conn_manager.clone();
    let prefix = &app_data.redis_prefix;

    // Validate state
    let code_verifier = oauth::validate_state(redis, prefix, state).await?;

    // Exchange code for token
    let access_token = oauth::exchange_github_code(config, code).await?;

    // Get user info from GitHub
    let github_user =
        oauth::get_github_user_info(&access_token, &config.base_url).await?;
    let email = github_user.email.ok_or_else(|| {
        DomainError::new_bad_input_error(
            "GitHub did not return an email".to_owned(),
        )
    })?;

    // Find or create user
    let app_data_clone = app_data.clone();
    let (user, _is_new) = web::block(move || {
        let pool = &app_data_clone.pool;
        let mut conn = pool.get()?;
        find_or_create_oauth_user(
            &email,
            &OAuthProvider::Github,
            &github_user.id.to_string(),
            github_user.name.as_deref(),
            app_data_clone.config.hash_cost,
            &app_data_clone.user_ids_cache,
            &mut conn,
        )
    })
    .await??;

    // Issue JWT and session
    issue_oauth_session(app_data, user, code_verifier).await
}

#[utoipa::path(
    get,
    path = "/api/auth/oauth/google/login",
    tag = "oauth",
    responses(
        (status = 307, description = "Redirects to Google OAuth"),
        (status = 401, description = "OAuth is not enabled", body = crate::models::misc::ErrorResponse<String>),
    ),
)]
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn google_login(
    app_data: Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    if !app_data.config.oauth.enabled {
        return Err(DomainError::new_auth_error(
            "OAuth is not enabled".to_owned(),
        ));
    }

    let redis = app_data.redis_conn_manager.clone();
    let prefix = &app_data.redis_prefix;

    let (state, code_challenge) =
        oauth::generate_state_and_challenge(redis, prefix).await?;

    let authorize_url = oauth::build_google_authorize_url(
        &app_data.config.oauth,
        &state,
        &code_challenge,
    )?;

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", authorize_url))
        .finish())
}

#[utoipa::path(
    get,
    path = "/api/auth/oauth/google/callback",
    tag = "oauth",
    responses(
        (status = 307, description = "Redirects to app root after successful login"),
        (status = 400, description = "Invalid OAuth callback parameters", body = crate::models::misc::ErrorResponse<String>),
    ),
)]
#[tracing::instrument(level = "info", skip(app_data, query))]
pub async fn google_callback(
    app_data: Data<AppData>,
    query: web::Query<OAuthCallbackQuery>,
) -> Result<HttpResponse, DomainError> {
    if let Some(ref error) = query.error {
        tracing::warn!(error = %error, "OAuth error from Google");
        return Ok(HttpResponse::BadRequest()
            .content_type("application/json")
            .body(format!("{{\"error\":\"{}\"}}", error)));
    }

    let state = query.state.as_deref().ok_or_else(|| {
        DomainError::new_bad_input_error("Missing state parameter".to_owned())
    })?;

    let code = query.code.as_deref().ok_or_else(|| {
        DomainError::new_bad_input_error("Missing code parameter".to_owned())
    })?;

    let config = &app_data.config.oauth;
    if !config.enabled {
        return Err(DomainError::new_auth_error(
            "OAuth is not enabled".to_owned(),
        ));
    }

    let redis = app_data.redis_conn_manager.clone();
    let prefix = &app_data.redis_prefix;

    // Validate state
    let code_verifier = oauth::validate_state(redis, prefix, state).await?;

    // Exchange code for token
    let access_token = oauth::exchange_google_code(config, code).await?;

    // Get user info from Google
    let google_user =
        oauth::get_google_user_info(&access_token, &config.base_url).await?;
    let email = google_user.email.clone();

    // Find or create user
    let app_data_clone = app_data.clone();
    let (user, _is_new) = web::block(move || {
        let pool = &app_data_clone.pool;
        let mut conn = pool.get()?;
        find_or_create_oauth_user(
            &email,
            &OAuthProvider::Google,
            &google_user.sub,
            google_user.name.as_deref(),
            app_data_clone.config.hash_cost,
            &app_data_clone.user_ids_cache,
            &mut conn,
        )
    })
    .await??;

    // Issue JWT and session
    issue_oauth_session(app_data, user, code_verifier).await
}

async fn issue_oauth_session(
    app_data: Data<AppData>,
    user: crate::models::users::UserWithRoles,
    _code_verifier: String,
) -> Result<HttpResponse, DomainError> {
    let credentials_repo = &app_data.credentials_repo;
    let jwt_key = &app_data.jwt_key;

    let session_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    let auth_data = VerifiedAuthDetails {
        user_id: user.id,
        session_id,
        username: user.username.clone(),
        roles: user.roles.clone(),
        device_id: device_id.to_string(),
    };

    let claims = Claims::with_custom_claims(auth_data, Duration::from_days(30));
    let token = jwt_key
        .authenticate(claims)
        .map_err(|err| DomainError::anyhow_auth("Failed to create JWT", err))?;

    // Create session info
    let now = Utc::now().naive_utc();
    let ttl_seconds = app_data.config.session.expiration_secs;

    let session_info = SessionInfo {
        session_id,
        device_id,
        device_name: None,
        created_at: now,
        last_used_at: now,
        token: token.clone(),
        ttl_remaining: Some(ttl_seconds as i64),
    };

    credentials_repo
        .create_session(&user.id, &session_id, &session_info, ttl_seconds)
        .await?;

    let cookie = Cookie::build("X-AUTH-TOKEN", &token)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .finish();

    tracing::info!(
        user_id = %user.id,
        username = %user.username,
        "OAuth login successful"
    );

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", "/"))
        .cookie(cookie)
        .finish())
}
