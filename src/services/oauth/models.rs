use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubOAuthUser {
    pub id: u64,
    pub login: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleOAuthUser {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: String,
    pub token_type: String,
    pub id_token: Option<String>,
}
