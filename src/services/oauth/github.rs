use super::models::{GitHubEmail, GitHubOAuthUser, GitHubTokenResponse};
use crate::config::OAuthProviderConfig;
use crate::errors::DomainError;
use serde::Serialize;
use url::Url;

const GITHUB_AUTH_PATH: &str = "/login/oauth/authorize";
const GITHUB_TOKEN_PATH: &str = "/login/oauth/access_token";
const GITHUB_USER_INFO_PATH: &str = "/user";
const GITHUB_EMAILS_PATH: &str = "/user/emails";

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
    let mut url = Url::parse(&format!("https://github.com{GITHUB_AUTH_PATH}"))
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to parse GitHub auth URL: {err}"
            ))
        })?;

    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair(
            "redirect_uri",
            &format!("{base_url}/api/v1/auth/oauth/github/callback"),
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
    github_base_url: &str,
    code_verifier: &str,
) -> Result<GitHubTokenResponse, DomainError> {
    let client = reqwest::Client::new();
    let redirect_uri = format!("{base_url}/api/v1/auth/oauth/github/callback");
    let token_url = format!("{github_base_url}{GITHUB_TOKEN_PATH}");

    let response = client
        .post(&token_url)
        .header("Accept", "application/json")
        .header("User-Agent", "actix-demo")
        .basic_auth(
            config.client_id.clone(),
            Some(config.client_secret.clone()),
        )
        .form(&serde_json::json!({
            "code": code,
            "redirect_uri": redirect_uri,
            "grant_type": "authorization_code",
            "code_verifier": code_verifier,
        }))
        .send()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to exchange OAuth code: {err}"
            ))
        })?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        tracing::error!(status = %status, body = %body, "GitHub token exchange failed");
        return Err(DomainError::new_internal_error(format!(
            "GitHub token exchange failed ({}): {}",
            status, body
        )));
    }
    tracing::info!(body = %body, "GitHub token exchange response");

    // Parse URL-encoded response: access_token=xxx&scope=yyy&token_type=zzz
    let token_response = parse_github_token_response(&body)?;
    Ok(token_response)
}

fn parse_github_token_response(
    body: &str,
) -> Result<GitHubTokenResponse, DomainError> {
    // Try JSON first (GitHub returns JSON when Accept: application/json)
    if let Ok(response) = serde_json::from_str::<GitHubTokenResponse>(body) {
        return Ok(response);
    }

    // Fallback to URL-encoded format
    let mut access_token = String::new();
    let mut scope = String::new();
    let mut token_type = String::new();

    for pair in body.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        match key {
            "access_token" => {
                access_token =
                    urlencoding::decode(value).unwrap_or_default().to_string()
            }
            "scope" => {
                scope =
                    urlencoding::decode(value).unwrap_or_default().to_string()
            }
            "token_type" => {
                token_type =
                    urlencoding::decode(value).unwrap_or_default().to_string()
            }
            _ => {}
        }
    }

    Ok(GitHubTokenResponse {
        access_token,
        scope,
        token_type,
    })
}

pub async fn get_user_info(
    access_token: &str,
    github_api_base_url: &str,
) -> Result<GitHubOAuthUser, DomainError> {
    let client = reqwest::Client::new();
    let user_url = format!("{github_api_base_url}{GITHUB_USER_INFO_PATH}");

    let response = client
        .get(&user_url)
        .header("Authorization", format!("token {access_token}"))
        .header("Accept", "application/json")
        .header("User-Agent", "actix-demo")
        .send()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to fetch GitHub user info: {err}"
            ))
        })?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    tracing::info!(status = %status, body = %body, "GitHub user info response");

    if !status.is_success() {
        return Err(DomainError::new_internal_error(format!(
            "GitHub user info failed ({}): {}",
            status, body
        )));
    }

    let user: GitHubOAuthUser = serde_json::from_str(&body).map_err(|err| {
        DomainError::new_internal_error(format!(
            "Failed to parse GitHub user info: {err}"
        ))
    })?;

    Ok(user)
}

pub async fn get_user_emails(
    access_token: &str,
    github_api_base_url: &str,
) -> Result<Vec<GitHubEmail>, DomainError> {
    let client = reqwest::Client::new();
    let emails_url = format!("{github_api_base_url}{GITHUB_EMAILS_PATH}");

    let emails = client
        .get(&emails_url)
        .header("Authorization", format!("token {access_token}"))
        .header("Accept", "application/json")
        .header("User-Agent", "actix-demo")
        .send()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to fetch GitHub emails: {err}"
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
