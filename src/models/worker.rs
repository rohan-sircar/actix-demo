use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct WorkerBackoffConfig {
    pub initial_interval_secs: u64,
    pub multiplier: f64,
    pub max_interval_secs: u64,
    pub max_elapsed_time_secs: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WorkerConfig {
    pub backoff: WorkerBackoffConfig,
    pub run_interval: u16,
}
