use std::{collections::HashMap, time::SystemTime};

use crate::{
    get_build_info,
    health::{HealthChecker, HealthcheckName},
    AppData,
};
use actix_web::{web::Data, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Healthy,
    Unhealthy(String),
}

#[derive(Serialize)]
struct HealthCheckResponse {
    version: String,
    timestamp: String,
    uptime: std::time::Duration,
    success: bool,
    services: HashMap<String, ServiceStatus>,
}

fn get_health_checkers(
    data: &Data<AppData>,
) -> Result<&[(HealthcheckName, HealthChecker)], ServiceStatus> {
    data.health_checkers.as_deref().ok_or_else(|| {
        ServiceStatus::Unhealthy("Health checkers not initialized".to_string())
    })
}

pub async fn healthcheck(data: Data<AppData>) -> impl Responder {
    let start_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    let bi = get_build_info();

    let checkers = get_health_checkers(&data)
        .expect("health checkers should be initialized");

    let check_futures = checkers.into_iter().map(|(service_name, checker)| {
        let service_name = service_name.to_string();
        let timeout_duration = std::time::Duration::from_secs(
            data.config.health_check_timeout_secs.into(),
        );

        async move {
            let check_result = tokio::time::timeout(
                timeout_duration,
                checker.check_health(timeout_duration),
            )
            .await;

            match check_result {
                Ok(Ok(_)) => (service_name, ServiceStatus::Healthy),
                Ok(Err(err)) => {
                    let _ = tracing::warn!("Health check failed: {err}");
                    (service_name, ServiceStatus::Unhealthy(err.to_string()))
                }
                Err(_) => {
                    let err_msg = format!(
                        "Timeout after {} seconds",
                        timeout_duration.as_secs()
                    );
                    let _ = tracing::warn!(err_msg);
                    (service_name, ServiceStatus::Unhealthy(err_msg))
                }
            }
        }
    });

    let check_results = futures::future::join_all(check_futures).await;

    let (services, success) = check_results.into_iter().fold(
        (HashMap::new(), true),
        |(mut acc, all_healthy), (name, status)| {
            let healthy = matches!(status, ServiceStatus::Healthy);
            acc.insert(name, status);
            (acc, all_healthy && healthy)
        },
    );

    let response = HealthCheckResponse {
        version: bi.crate_info.version.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime: start_time,
        success,
        services,
    };

    if success {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}
