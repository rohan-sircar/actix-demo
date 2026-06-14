use crate::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TlsMode {
    #[default]
    None,
    #[serde(rename = "starttls")]
    StartTls,
    Tls,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OAuthConfig {
    pub enabled: bool,
    pub base_url: String,
    pub github_base_url: String,
    pub github_api_base_url: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    #[serde(default)]
    pub github_scopes: Vec<String>,
    pub google_client_id: String,
    pub google_client_secret: String,
    #[serde(default)]
    pub google_scopes: Vec<String>,
}

impl OAuthConfig {
    pub fn github(&self) -> OAuthProviderConfig {
        OAuthProviderConfig {
            client_id: self.github_client_id.clone(),
            client_secret: self.github_client_secret.clone(),
            scopes: self.github_scopes.clone(),
        }
    }

    pub fn github_api_base_url(&self) -> String {
        self.github_api_base_url.clone()
    }

    pub fn google(&self) -> OAuthProviderConfig {
        OAuthProviderConfig {
            client_id: self.google_client_id.clone(),
            client_secret: self.google_client_secret.clone(),
            scopes: self.google_scopes.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OAuthProviderConfig {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfig {
    // system
    pub loki_url: url::Url,
    pub prometheus_url: url::Url,
    pub database_url: String,
    pub http_host: String,
    #[serde(default = "models::defaults::default_http_port")]
    pub http_port: u16,
    #[serde(default = "models::defaults::default_app_base_url")]
    pub app_base_url: String,
    #[serde(default = "models::defaults::default_hash_cost")]
    pub hash_cost: u32,
    pub logger_format: LoggerFormat,
    pub jwt_key: String,
    pub redis_url: String,
    pub job_bin_path: String,
    #[serde(
        default = "models::defaults::default_rate_limit_auth_max_requests"
    )]
    // rate limit
    pub rate_limit_auth_max_requests: u32,
    #[serde(default = "models::defaults::default_rate_limit_auth_window_secs")]
    pub rate_limit_auth_window_secs: u64,
    #[serde(default = "models::defaults::default_rate_limit_api_max_requests")]
    pub rate_limit_api_max_requests: u32,
    #[serde(default = "models::defaults::default_rate_limit_api_window_secs")]
    pub rate_limit_api_window_secs: u64,
    #[serde(
        default = "models::defaults::default_rate_limit_api_public_max_requests"
    )]
    pub rate_limit_api_public_max_requests: u32,
    #[serde(
        default = "models::defaults::default_rate_limit_api_public_window_secs"
    )]
    pub rate_limit_api_public_window_secs: u64,
    pub rate_limit_disable: bool,
    // session
    #[serde(default = "models::defaults::default_session_expiration_secs")]
    pub session_expiration_secs: u64,
    #[serde(
        default = "models::defaults::default_session_cleanup_interval_secs"
    )]
    pub session_cleanup_interval_secs: u16,
    #[serde(default = "models::defaults::default_max_concurrent_sessions")]
    pub max_concurrent_sessions: usize,
    #[serde(default = "models::defaults::default_session_renewal_enabled")]
    pub session_renewal_enabled: bool,
    #[serde(default = "models::defaults::default_session_renewal_window_secs")]
    pub session_renewal_window_secs: u64,
    #[serde(default = "models::defaults::default_session_max_renewals")]
    pub session_max_renewals: u32,
    #[serde(default)]
    pub session_disable: bool,
    // worker
    #[serde(
        default = "models::defaults::default_worker_initial_interval_secs"
    )]
    pub worker_initial_interval_secs: u64,
    #[serde(default = "models::defaults::default_worker_multiplier")]
    pub worker_multiplier: f64,
    #[serde(default = "models::defaults::default_worker_max_interval_secs")]
    pub worker_max_interval_secs: u64,
    #[serde(
        default = "models::defaults::default_worker_max_elapsed_time_secs"
    )]
    pub worker_max_elapsed_time_secs: u64,
    #[serde(default = "models::defaults::default_worker_run_interval_secs")]
    pub worker_run_interval_secs: u8,
    #[serde(default = "models::defaults::default_health_check_timeout_secs")]
    pub health_check_timeout_secs: u8,
    // MinIO configuration
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_secure: bool,
    pub minio_bucket_name: String,
    #[serde(default = "default_avatar_size_limit")]
    pub max_avatar_size_bytes: u64,
    #[serde(default = "default_pet_image_size_limit")]
    pub max_pet_image_size_bytes: u64,
    #[serde(default = "models::defaults::default_timezone")]
    pub timezone: chrono_tz::Tz,
    // SMTP configuration
    #[serde(default = "models::defaults::default_smtp_host")]
    pub smtp_host: String,
    #[serde(default = "models::defaults::default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default = "models::defaults::default_smtp_username")]
    pub smtp_username: String,
    #[serde(default = "models::defaults::default_smtp_password")]
    pub smtp_password: String,
    #[serde(default = "models::defaults::default_smtp_from_email")]
    pub smtp_from_email: String,
    #[serde(default = "models::defaults::default_tls_mode")]
    pub smtp_tls_mode: TlsMode,
    // Email token TTLs
    #[serde(
        default = "models::defaults::default_email_token_ttl_verification_secs"
    )]
    pub email_token_ttl_verification_secs: u64,
    #[serde(default = "models::defaults::default_email_token_ttl_reset_secs")]
    pub email_token_ttl_reset_secs: u64,
    // Link templates
    #[serde(default = "models::defaults::default_verification_link_template")]
    pub verification_link_template: String,
    #[serde(
        default = "models::defaults::default_password_reset_link_template"
    )]
    pub password_reset_link_template: String,
    // Rate limiting for registration and password reset
    #[serde(
        default = "models::defaults::default_rate_limit_registration_max_requests"
    )]
    pub rate_limit_registration_max_requests: u32,
    #[serde(
        default = "models::defaults::default_rate_limit_registration_window_secs"
    )]
    pub rate_limit_registration_window_secs: u64,
    #[serde(
        default = "models::defaults::default_rate_limit_password_reset_max_requests"
    )]
    pub rate_limit_password_reset_max_requests: u32,
    #[serde(
        default = "models::defaults::default_rate_limit_password_reset_window_secs"
    )]
    pub rate_limit_password_reset_window_secs: u64,
    // OAuth configuration (flat for envy compatibility)
    #[serde(default = "models::defaults::default_oauth_enabled")]
    pub oauth_enabled: bool,
    #[serde(default = "models::defaults::default_oauth_base_url")]
    pub oauth_base_url: String,
    #[serde(default = "models::defaults::default_oauth_github_base_url")]
    pub oauth_github_base_url: String,
    #[serde(default = "models::defaults::default_oauth_github_api_base_url")]
    pub oauth_github_api_base_url: String,
    #[serde(default)]
    pub oauth_github_client_id: String,
    #[serde(default)]
    pub oauth_github_client_secret: String,
    #[serde(default)]
    pub oauth_github_scopes: String,
    #[serde(default)]
    pub oauth_google_client_id: String,
    #[serde(default)]
    pub oauth_google_client_secret: String,
    #[serde(default)]
    pub oauth_google_scopes: String,
    // API documentation path
    #[serde(default = "models::defaults::default_swagger_path")]
    pub swagger_path: String,
    // CORS allowed origins (comma-separated)
    #[serde(default = "models::defaults::default_cors_origins")]
    pub cors_origins: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MinioConfig {
    // Bucket name for avatars
    pub bucket_name: String,

    // Maximum avatar size in bytes
    #[serde(default = "default_avatar_size_limit")]
    pub max_avatar_size_bytes: u64,

    // Maximum pet image size in bytes
    #[serde(default = "default_pet_image_size_limit")]
    pub max_pet_image_size_bytes: u64,
}

pub fn default_avatar_size_limit() -> u64 {
    2 * 1024 * 1024 // 2MB
}

pub fn default_pet_image_size_limit() -> u64 {
    5 * 1024 * 1024 // 5MB
}
