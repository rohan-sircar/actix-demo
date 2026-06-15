pub fn default_hash_cost() -> u32 {
    8
}

pub fn default_rate_limit_auth_max_requests() -> u32 {
    5
}

pub fn default_rate_limit_auth_window_secs() -> u64 {
    120
}

pub fn default_rate_limit_api_max_requests() -> u32 {
    500
}

pub fn default_rate_limit_api_window_secs() -> u64 {
    60
}

pub fn default_rate_limit_api_public_max_requests() -> u32 {
    15
}

pub fn default_rate_limit_api_public_window_secs() -> u64 {
    60
}

pub fn default_session_expiration_secs() -> u64 {
    86400
}

pub fn default_session_cleanup_interval_secs() -> u16 {
    600
}

pub fn default_max_concurrent_sessions() -> usize {
    5
}

pub fn default_session_renewal_enabled() -> bool {
    true
}

pub fn default_session_renewal_window_secs() -> u64 {
    1800
}

pub fn default_session_max_renewals() -> u32 {
    3
}

pub fn default_worker_initial_interval_secs() -> u64 {
    3
}

pub fn default_worker_multiplier() -> f64 {
    2.0
}

pub fn default_worker_max_interval_secs() -> u64 {
    30
}

pub fn default_worker_max_elapsed_time_secs() -> u64 {
    300
}

pub fn default_worker_run_interval_secs() -> u8 {
    10
}
pub fn default_health_check_timeout_secs() -> u8 {
    10
}
pub fn default_timezone() -> chrono_tz::Tz {
    chrono_tz::Tz::UTC
}

pub fn default_smtp_host() -> String {
    "localhost".to_string()
}

pub fn default_smtp_port() -> u16 {
    587
}

pub fn default_smtp_username() -> String {
    "".to_string()
}

pub fn default_smtp_password() -> String {
    "".to_string()
}

pub fn default_smtp_from_email() -> String {
    "noreply@example.com".to_string()
}

pub fn default_tls_mode() -> crate::config::TlsMode {
    crate::config::TlsMode::None
}

pub fn default_email_token_ttl_verification_secs() -> u64 {
    86400 // 24 hours
}

pub fn default_email_token_ttl_reset_secs() -> u64 {
    900 // 15 minutes
}

pub fn default_http_port() -> u16 {
    8800
}

pub fn default_app_base_url() -> String {
    "http://localhost:8800".to_string()
}

pub fn default_verification_link_template() -> String {
    "{base_url}/verify?token={token}&user={user_name}".to_string()
}

pub fn default_password_reset_link_template() -> String {
    "{base_url}/reset-password?token={token}&user={user_name}"
        .to_string()
}

pub fn default_mobile_verification_link_template() -> String {
    "my-expo-app://verify?token={token}&user={user_name}".to_string()
}

pub fn default_rate_limit_registration_max_requests() -> u32 {
    3
}

pub fn default_rate_limit_registration_window_secs() -> u64 {
    3600 // 1 hour
}

pub fn default_rate_limit_password_reset_max_requests() -> u32 {
    5
}

pub fn default_rate_limit_password_reset_window_secs() -> u64 {
    300 // 5 minutes
}

pub fn default_oauth_enabled() -> bool {
    true
}

pub fn default_oauth_base_url() -> String {
    "http://localhost:8800".to_string()
}

pub fn default_oauth_github_base_url() -> String {
    "https://github.com".to_string()
}

pub fn default_oauth_github_api_base_url() -> String {
    "https://api.github.com".to_string()
}

pub fn default_oauth_config() -> crate::config::OAuthConfig {
    crate::config::OAuthConfig {
        enabled: default_oauth_enabled(),
        base_url: default_oauth_base_url(),
        github_base_url: default_oauth_github_base_url(),
        github_api_base_url: default_oauth_github_api_base_url(),
        github_client_id: String::new(),
        github_client_secret: String::new(),
        github_scopes: Vec::new(),
        google_client_id: String::new(),
        google_client_secret: String::new(),
        google_scopes: Vec::new(),
    }
}

pub fn default_swagger_path() -> String {
    "/swagger".to_string()
}

pub fn default_cors_origins() -> String {
    "*".to_string()
}
