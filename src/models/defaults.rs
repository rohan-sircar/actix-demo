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

pub fn default_session_expiration_secs() -> u64 {
    86400
}

pub fn default_session_cleanup_interval_secs() -> u64 {
    600
}

pub fn default_max_concurrent_sessions() -> u32 {
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
