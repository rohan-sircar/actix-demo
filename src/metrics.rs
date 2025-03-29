use prometheus::{opts, GaugeVec, IntCounterVec, Registry};

#[derive(Clone)]
pub struct Metrics {
    pub job_counter: IntCounterVec,
    pub active_sessions: GaugeVec,
}

impl Metrics {
    pub fn new(registry: Registry) -> Self {
        let job_counter = IntCounterVec::new(
            opts!("api_jobs_total", "Total job executions"),
            &["status"], // running, completed, aborted
        )
        .unwrap();

        let active_sessions = GaugeVec::new(
            opts!(
                "api_active_sessions_total",
                "Currently active user sessions"
            ),
            &["user_id"],
        )
        .unwrap();

        registry.register(Box::new(job_counter.clone())).unwrap();
        registry
            .register(Box::new(active_sessions.clone()))
            .unwrap();

        Self {
            job_counter,
            active_sessions,
        }
    }
}
