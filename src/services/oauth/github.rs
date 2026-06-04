use super::models::{GitHubEmail, GitHubOAuthUser, GitHubTokenResponse};
use crate::config::OAuthProviderConfig;
use crate::errors::DomainError;
use serde::Serialize;
use url::Url;

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_INFO_URL: &str = "https://api.github.com/user";
const GITHUB_EMAILS_URL: &str = "https://api.github.com/user/emails";

#[derive(Serialize)]
pub struct GitHubAuthUrlParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub response_type: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

pub fn build_authorize_url(
    config: &OAuthProviderConfig,
    base_url: &str,
    state: &str,
    code_challenge: &str,
) -> Result<String, DomainError> {
    let mut url = Url::parse(GITHUB_AUTH_URL).map_err(|err| {
        DomainError::new_internal_error(format!(
            "Failed to parse GitHub auth URL: {err}"
        ))
    })?;

    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair(
            "redirect_uri",
            &format!("{base_url}/api/auth/oauth/github/callback"),
        )
        .append_pair("scope", "read:user user:email")
        .append_pair("response_type", "code")
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256");

    Ok(url.to_string())
}

pub async fn exchange_code_for_token(
    config: &OAuthProviderConfig,
    code: &str,
    base_url: &str,
) -> Result<GitHubTokenResponse, DomainError> {
    let client = reqwest::Client::new();
    let redirect_uri = format!("{base_url}/api/auth/oauth/github/callback");

    let response = client
        .post(GITHUB_TOKEN_URL)
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
            "GitHub token exchange failed: {body}"
        )));
    }

    let token_response: GitHubTokenResponse =
        response.json().await.map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to parse GitHub token response: {err}"
            ))
        })?;

    Ok(token_response)
}

pub async fn get_user_info(
    access_token: &str,
) -> Result<GitHubOAuthUser, DomainError> {
    let client = reqwest::Client::new();

    let user = client
        .get(GITHUB_USER_INFO_URL)
        .header("Authorization", format!("token {access_token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to fetch GitHub user info: {err}"
            ))
        })?
        .json::<GitHubOAuthUser>()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to parse GitHub user info: {err}"
            ))
        })?;

    Ok(user)
}

pub async fn get_user_emails(
    access_token: &str,
) -> Result<Vec<GitHubEmail>, DomainError> {
    let client = reqwest::Client::new();

    let emails = client
        .get(GITHUB_EMAILS_URL)
        .header("Authorization", format!("token {access_token}"))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to fetch GitHub user emails: {err}"
            ))
        })?
        .json::<Vec<GitHubEmail>>()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to parse GitHub emails: {err}"
            ))
        })?;

    Ok(emails)
}
