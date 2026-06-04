use super::models::{GoogleOAuthUser, GoogleTokenResponse};
use crate::config::OAuthProviderConfig;
use crate::errors::DomainError;
use serde::Serialize;
use url::Url;

const GOOGLE_AUTH_PATH: &str = "/o/oauth2/v2/auth";
const GOOGLE_TOKEN_PATH: &str = "/oauth2/v4/token";
const GOOGLE_USER_INFO_PATH: &str = "/oauth2/v2/userinfo";

#[derive(Serialize)]
pub struct GoogleAuthUrlParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub response_type: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub access_type: String,
    pub prompt: String,
}

pub fn build_authorize_url(
    config: &OAuthProviderConfig,
    base_url: &str,
    state: &str,
    code_challenge: &str,
) -> Result<String, DomainError> {
    let auth_base = if base_url.is_empty() || base_url.starts_with("https://accounts.google.com") {
        "https://accounts.google.com"
    } else {
        base_url
    };
    let mut url = Url::parse(&format!("{auth_base}{GOOGLE_AUTH_PATH}")).map_err(|err| {
        DomainError::new_internal_error(format!(
            "Failed to parse Google auth URL: {err}"
        ))
    })?;

    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair(
            "redirect_uri",
            &format!("{base_url}/api/auth/oauth/google/callback"),
        )
        .append_pair("scope", "openid email profile")
        .append_pair("response_type", "code")
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");

    Ok(url.to_string())
}

pub async fn exchange_code_for_token(
    config: &OAuthProviderConfig,
    code: &str,
    base_url: &str,
) -> Result<GoogleTokenResponse, DomainError> {
    let client = reqwest::Client::new();
    let redirect_uri = format!("{base_url}/api/auth/oauth/google/callback");
    let token_url = format!("{base_url}{GOOGLE_TOKEN_PATH}");

    let response = client
        .post(&token_url)
        .header("Accept", "application/json")
        .form(&serde_json::json!({
            "client_id": config.client_id,
            "client_secret": config.client_secret,
            "code": code,
            "redirect_uri": redirect_uri,
            "grant_type": "authorization_code",
        }))
        .send()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to exchange OAuth code: {err}"
            ))
        })?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(DomainError::new_internal_error(format!(
            "Google token exchange failed: {body}"
        )));
    }

    let token_response: GoogleTokenResponse =
        response.json().await.map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to parse Google token response: {err}"
            ))
        })?;

    Ok(token_response)
}

pub async fn get_user_info(
    access_token: &str,
    base_url: &str,
) -> Result<GoogleOAuthUser, DomainError> {
    let client = reqwest::Client::new();
    let user_url = format!("{base_url}{GOOGLE_USER_INFO_PATH}");

    let user = client
        .get(&user_url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to fetch Google user info: {err}"
            ))
        })?
        .json::<GoogleOAuthUser>()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to parse Google user info: {err}"
            ))
        })?;

    Ok(user)
}
