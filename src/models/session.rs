use derive_builder::Builder;
use serde::Deserialize;

/// Configuration for session management
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "snake_case")]
pub struct SessionConfig {
    /// Session expiration time in seconds
    #[builder(default = "86400")] // 24 hours
    pub expiration_secs: u64,
    /// Session renewal policy configuration
    #[builder(
        default = "SessionRenewalPolicyBuilder::default().build().unwrap()"
    )]
    pub renewal: SessionRenewalPolicy,
    /// Session cleanup interval in seconds
    #[builder(default = "600")] // 10 minutes
    pub cleanup_interval_secs: u64,
    /// Maximum number of concurrent sessions per user
    #[builder(default = "5")]
    pub max_concurrent_sessions: u8,
    /// Whether session management is disabled
    #[builder(default = "false")]
    pub disable: bool,
}

/// Policy configuration for session renewal
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct SessionRenewalPolicy {
    /// Whether session renewal is enabled
    #[builder(default = "true")]
    pub enabled: bool,
    /// Time window in seconds before expiration when renewal is allowed
    #[builder(default = "1800")] // 30 minutes
    pub renewal_window_secs: u64,
    /// Maximum number of renewals allowed per session
    #[builder(default = "3")]
    pub max_renewals: u32,
}
