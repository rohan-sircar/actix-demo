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
    let auth_base = if base_url.is_empty() || base_url == "https://github.com" {
        "https://github.com"
    } else {
        base_url
    };
    let mut url = Url::parse(&format!("{auth_base}{GITHUB_AUTH_PATH}")).map_err(|err| {
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
    let token_url = format!("{base_url}{GITHUB_TOKEN_PATH}");

    let response = client
        .post(&token_url)
        .header("Accept", "application/json")
        .basic_auth(config.client_id.clone(), Some(config.client_secret.clone()))
        .form(&serde_json::json!({
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

    let body = response.text().await.map_err(|err| {
        DomainError::new_internal_error(format!(
            "Failed to read GitHub token response: {err}"
        ))
    })?;

    // Parse URL-encoded response: access_token=xxx&scope=yyy&token_type=zzz
    let token_response = parse_github_token_response(&body)?;
    Ok(token_response)
}

fn parse_github_token_response(body: &str) -> Result<GitHubTokenResponse, DomainError> {
    let mut access_token = String::new();
    let mut scope = String::new();
    let mut token_type = String::new();

    for pair in body.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        match key {
            "access_token" => access_token = urlencoding::decode(value).unwrap_or_default().to_string(),
            "scope" => scope = urlencoding::decode(value).unwrap_or_default().to_string(),
            "token_type" => token_type = urlencoding::decode(value).unwrap_or_default().to_string(),
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
    base_url: &str,
) -> Result<GitHubOAuthUser, DomainError> {
    let client = reqwest::Client::new();
    let user_url = format!("{base_url}{GITHUB_USER_INFO_PATH}");

    let user = client
        .get(&user_url)
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
    base_url: &str,
) -> Result<Vec<GitHubEmail>, DomainError> {
    let client = reqwest::Client::new();
    let emails_url = format!("{base_url}{GITHUB_EMAILS_PATH}");

    let emails = client
        .get(&emails_url)
        .header("Authorization", format!("token {access_token}"))
        .header("Accept", "application/json")
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
