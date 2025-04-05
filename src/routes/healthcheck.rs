use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use crate::{get_build_info, AppData};
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
    uptime: u64,
    success: bool,
    services: HashMap<String, ServiceStatus>,
}

#[tracing::instrument(level = "info", skip_all)]
pub async fn healthcheck(app_data: Data<AppData>) -> impl Responder {
    let uptime = SystemTime::now()
        .duration_since(app_data.start_time)
        .unwrap_or_default()
        .as_secs();

    let bi = get_build_info();

    let checkers = &app_data.health_checkers;

    let check_futures = checkers.iter().map(|(service_name, checker)| {
        let service_name = service_name.to_string();
        let timeout = Duration::from_secs(
            app_data.config.health_check_timeout_secs.into(),
        );

        async move {
            checker
                .check_health(timeout)
                .await
                .map(|_| (service_name.clone(), ServiceStatus::Healthy))
                .map_err(|err| {
                    let _ = tracing::warn!(
                        "Health check failed for {service_name}: {err}"
                    );
                    (service_name, ServiceStatus::Unhealthy(err.to_string()))
                })
        }
    });

    let check_results = futures::future::join_all(check_futures).await;

    let (services, success) = check_results.into_iter().fold(
        (HashMap::new(), true),
        |(mut acc, all_healthy), result| match result {
            Ok((name, status)) => {
                let healthy = matches!(status, ServiceStatus::Healthy);
                acc.insert(name, status);
                (acc, all_healthy && healthy)
            }
            Err((name, status)) => {
                acc.insert(name, status);
                (acc, false)
            }
        },
    );

    let response = HealthCheckResponse {
        version: bi.crate_info.version.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime,
        success,
        services,
    };

    if success {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}
