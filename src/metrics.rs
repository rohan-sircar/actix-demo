use lazy_static::lazy_static;
use prometheus::{opts, GaugeVec, IntCounterVec};

lazy_static! {
    // Job metrics
    pub static ref JOB_COUNTER: IntCounterVec = IntCounterVec::new(
        opts!("jobs_total", "Total job executions"),
        &["status"] // running, completed, aborted
    ).unwrap();

    // Session metrics
    pub static ref ACTIVE_SESSIONS: GaugeVec = GaugeVec::new(
        opts!("active_sessions_total", "Currently active user sessions"),
        &["user_id"]
    ).unwrap();
}

/// Register all custom metrics with the default registry
pub fn register_custom_metrics() {
    let registry = prometheus::default_registry();

    registry.register(Box::new(JOB_COUNTER.clone())).unwrap();
    registry
        .register(Box::new(ACTIVE_SESSIONS.clone()))
        .unwrap();
}
