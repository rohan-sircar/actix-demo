use prometheus::{opts, GaugeVec, HistogramVec, IntCounterVec, Registry};

#[derive(Clone)]
pub struct Metrics {
    pub active_sessions: GaugeVec,
    pub cache: CacheMetrics,
}

impl Metrics {
    pub fn new(registry: Registry) -> Self {
        let job_counter = IntCounterVec::new(
            opts!("api_jobs_total", "Total job executions"),
            &["status"], // running, completed, aborted
        )
        .unwrap();

        let active_sessions = GaugeVec::new(
            opts!("active_sessions_total", "Currently active user sessions"),
            &["user_id"],
        )
        .unwrap();

        registry.register(Box::new(job_counter.clone())).unwrap();
        registry
            .register(Box::new(active_sessions.clone()))
            .unwrap();

        Self {
            active_sessions,
            cache: CacheMetrics::new(&registry),
        }
    }
}

#[derive(Clone)]
pub struct CacheMetrics {
    pub hits: IntCounterVec,
    pub misses: IntCounterVec,
    pub errors: IntCounterVec,
    pub latency: HistogramVec,
}

impl CacheMetrics {
    pub fn new(registry: &Registry) -> Self {
        let hits = IntCounterVec::new(
            opts!("cache_hits_total", "Total cache hits"),
            &["cache_name"],
        )
        .unwrap();

        let misses = IntCounterVec::new(
            opts!("cache_misses_total", "Total cache misses"),
            &["cache_name"],
        )
        .unwrap();

        let errors = IntCounterVec::new(
            opts!("cache_errors_total", "Total cache errors"),
            &["cache_name"],
        )
        .unwrap();

        let latency = HistogramVec::new(
            opts!("cache_latency_seconds", "Cache operation latency").into(),
            &["cache_name", "operation"],
        )
        .unwrap();

        registry.register(Box::new(hits.clone())).unwrap();
        registry.register(Box::new(misses.clone())).unwrap();
        registry.register(Box::new(errors.clone())).unwrap();
        registry.register(Box::new(latency.clone())).unwrap();

        Self {
            hits,
            misses,
            errors,
            latency,
        }
    }
}
