# Metrics Enhancement Proposal

## 1. Core Metrics Implementation

### HTTP Request Tracking

```rust
// In src/lib.rs
use actix_web_prom::PrometheusMetrics;

pub fn configure_app(
    app_data: Data<AppData>,
) -> Box<dyn Fn(&mut ServiceConfig)> {
    let prometheus = PrometheusMetrics::new("api", "/metrics");

    Box::new(move |cfg: &mut ServiceConfig| {
        // Existing middleware configuration
        let login_limiter = rate_limit::create_login_rate_limiter(&app_data);
        let api_rate_limiter = || rate_limit::create_api_rate_limiter(&app_data);

        cfg.app_data(app_data.clone())
            .wrap(prometheus.clone())
            // Rest of existing configuration...
    })
}
```

### Job State Tracking

```rust
// In routes/command.rs
lazy_static! {
    static ref JOB_COUNTER: IntCounterVec = register_int_counter_vec!(
        "jobs_total",
        "Total job executions",
        &["status"]  // running, completed, aborted
    ).unwrap();
}

async fn handle_run_command(/*...*/) -> impl Responder {
    JOB_COUNTER.with_label_values(&["running"]).inc();
    // Job logic...
}
```

### Active Session Monitoring

```rust
// In models/session.rs
lazy_static! {
    static ref ACTIVE_SESSIONS: Gauge = register_gauge!(
        "active_sessions_total",
        "Currently active user sessions"
    ).unwrap();
}

pub async fn load_all_sessions(/*...*/) -> Result<...> {
    let sessions = /*...*/;
    ACTIVE_SESSIONS.set(sessions.len() as f64);
    Ok(sessions)
}
```

## 2. Recommended Additional Metrics

1. **Error Rates**

   - Track 4xx/5xx responses per endpoint
   - Database query failures

2. **System Metrics**

   - Memory usage
   - Thread pool utilization
   - TCP connection stats

3. **Business Metrics**
   - User registration rates
   - Concurrent websocket connections
   - Cache hit ratios

## 3. Implementation Steps

1. [x] Add crate dependency to Cargo.toml
2. [x] Configure Prometheus middleware in src/lib.rs
3. [x] Create metrics module with custom collectors
4. [x] Instrument key endpoints with metric macros
   1. [x] Add session count metrics
   2. [x] Add job status metrics
5. [] Add Grafana dashboard templates
6. [] Update deployment for metrics scraping
