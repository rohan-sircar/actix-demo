use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub max_concurrent_sessions: usize,
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
    /// Time window in seconds that session gets extended by on each successful authentication
    #[builder(default = "1800")] // 30 minutes
    pub renewal_window_secs: u64,
    /// Maximum number of renewals allowed per session
    #[builder(default = "3")]
    pub max_renewals: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub device_id: Uuid,
    pub device_name: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub last_used_at: chrono::NaiveDateTime,
    pub token: String,
    #[serde(skip)] // Skip serialization/deserialization
    pub ttl_remaining: Option<i64>,
}

#[derive(PartialEq)]
pub enum SessionStatus {
    Expired,
    Alive,
}

impl SessionStatus {
    pub fn from_exists(exists: bool) -> SessionStatus {
        if exists {
            SessionStatus::Alive
        } else {
            SessionStatus::Expired
        }
    }
}
