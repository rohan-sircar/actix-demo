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
