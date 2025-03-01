use serde::Deserialize;

/// Enum for the key strategy in rate limiting
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KeyStrategy {
    Ip,
    Random,
}

/// Configuration for rate limiting policies across different endpoints
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RateLimitConfig {
    /// Base key strategy for rate limiting ("ip" or "random")
    pub key_strategy: KeyStrategy,
    /// Authentication endpoint rate limiting policy
    pub auth: RateLimitPolicy,
    /// General API endpoint rate limiting policy
    pub api: RateLimitPolicy,
    // /// Redis-specific rate limiting configuration
    // pub redis: RedisRateLimitConfig,
}

/// Policy configuration for a rate-limited endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitPolicy {
    /// Maximum number of requests allowed within the window
    pub max_requests: u32,
    /// Time window in seconds for rate limiting
    pub window_secs: u64,
    // /// Optional burst capacity beyond the regular rate limit
    // #[serde(default)]
    // pub burst: Option<u32>,
    // /// Optional jitter window in seconds for burst requests
    // #[serde(default)]
    // pub jitter_secs: Option<u64>,
}

// /// Redis-specific configuration for rate limiting storage
// #[derive(Debug, Clone, Deserialize)]
// pub struct RedisRateLimitConfig {
//     /// Prefix for Redis keys used in rate limiting
//     pub key_prefix: String,
//     /// Time-to-live in seconds for Redis keys (should exceed window_secs)
//     pub key_ttl_secs: u64,
//     /// Whether Redis cluster mode is enabled
//     #[serde(default)]
//     pub cluster_mode: bool,
// }

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            key_strategy: KeyStrategy::Ip,
            auth: RateLimitPolicy {
                max_requests: 5,
                window_secs: 120, // 2 minutes
                                  // burst: Some(2),
                                  // jitter_secs: Some(1),
            },
            api: RateLimitPolicy {
                max_requests: 500,
                window_secs: 60,
                // burst: Some(20),
                // jitter_secs: Some(1),
            },
            // redis: RedisRateLimitConfig {
            //     key_prefix: "rate_limit".to_string(),
            //     key_ttl_secs: 600, // 10 minutes
            //     cluster_mode: false,
            // },
        }
    }
}
