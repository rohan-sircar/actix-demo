use crate::*;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfig {
    // system
    pub loki_url: url::Url,
    pub prometheus_url: url::Url,
    pub database_url: String,
    pub http_host: String,
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
    #[serde(default = "models::defaults::default_timezone")]
    pub timezone: chrono_tz::Tz,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MinioConfig {
    // Bucket name for avatars
    pub bucket_name: String,

    // Maximum avatar size in bytes
    #[serde(default = "default_avatar_size_limit")]
    pub max_avatar_size_bytes: u64,
}

pub fn default_avatar_size_limit() -> u64 {
    2 * 1024 * 1024 // 2MB
}
