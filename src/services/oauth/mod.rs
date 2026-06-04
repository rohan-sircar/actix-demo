pub mod github;
pub mod google;
pub mod models;

use crate::config::OAuthConfig;
use crate::errors::DomainError;
use crate::services::oauth::models::{GitHubOAuthUser, GoogleOAuthUser};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

pub async fn generate_state_and_challenge(
    mut redis: ConnectionManager,
    prefix: &dyn Fn(&dyn std::fmt::Display) -> String,
) -> Result<(String, String), DomainError> {
    use sha2::Digest;

    let state = Uuid::new_v4().to_string();
    let code_verifier = Uuid::new_v4().to_string();
    let mut hasher = sha2::Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = base64_url_encode(&hasher.finalize());

    let key = format!("{}{}", prefix(&"oauth:state"), state);
    redis
        .set_ex::<_, _, ()>(&key, &code_verifier, 300)
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to store OAuth state in Redis: {err}"
            ))
        })?;

    Ok((state, code_challenge))
}

pub fn base64_url_encode(data: &[u8]) -> String {
    use data_encoding::BASE64URL;
    BASE64URL.encode(data)
}

pub async fn validate_state(
    mut redis: ConnectionManager,
    prefix: &dyn Fn(&dyn std::fmt::Display) -> String,
    state: &str,
) -> Result<String, DomainError> {
    let key = format!("{}{}", prefix(&"oauth:state"), state);

    let code_verifier: Option<String> =
        redis.get(&key).await.map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to retrieve OAuth state from Redis: {err}"
            ))
        })?;

    match code_verifier {
        Some(verifier) => {
            let _: Result<usize, _> = redis.del(&key).await;
            Ok(verifier)
        }
        None => Err(DomainError::new_bad_input_error(
            "Session not found or expired".to_owned(),
        )),
    }
}

pub async fn get_github_user_info(
    access_token: &str,
    base_url: &str,
) -> Result<GitHubOAuthUser, DomainError> {
    let github_user = github::get_user_info(access_token, base_url).await?;
    let emails = github::get_user_emails(access_token, base_url).await?;

    let primary_email = emails
        .iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email.clone());

    if primary_email.is_none() {
        return Err(DomainError::new_bad_input_error(
            "GitHub did not return a verified email".to_owned(),
        ));
    }

    Ok(GitHubOAuthUser {
        id: github_user.id,
        login: github_user.login,
        email: primary_email,
        name: github_user.name,
        avatar_url: github_user.avatar_url,
    })
}

pub fn build_github_authorize_url(
    config: &OAuthConfig,
    state: &str,
    code_challenge: &str,
) -> Result<String, DomainError> {
    github::build_authorize_url(
        &config.github,
        &config.base_url,
        state,
        code_challenge,
    )
}

pub async fn exchange_github_code(
    config: &OAuthConfig,
    code: &str,
) -> Result<String, DomainError> {
    let token_response =
        github::exchange_code_for_token(&config.github, code, &config.base_url)
            .await?;
    Ok(token_response.access_token)
}

pub fn build_google_authorize_url(
    config: &OAuthConfig,
    state: &str,
    code_challenge: &str,
) -> Result<String, DomainError> {
    google::build_authorize_url(
        &config.google,
        &config.base_url,
        state,
        code_challenge,
    )
}

pub async fn exchange_google_code(
    config: &OAuthConfig,
    code: &str,
) -> Result<String, DomainError> {
    let token_response =
        google::exchange_code_for_token(&config.google, code, &config.base_url)
            .await?;
    Ok(token_response.access_token)
}

pub async fn get_google_user_info(
    access_token: &str,
    base_url: &str,
) -> Result<GoogleOAuthUser, DomainError> {
    google::get_user_info(access_token, base_url).await
}
