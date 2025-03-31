use std::{collections::HashMap, time::SystemTime};

use crate::{
    get_build_info,
    health::{HealthCheckable, HealthChecker, HealthcheckName},
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

    let mut services = HashMap::new();

    let mut success = true;
    for (service_name, checker) in checkers {
        let result = checker
            .check_health(std::time::Duration::from_secs(5))
            .await;

        let status = match result {
            Ok(_) => ServiceStatus::Healthy,
            Err(ref e) => {
                success = false;
                ServiceStatus::Unhealthy(e.to_string())
            }
        };

        services.insert(service_name.to_string(), status);
    }

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
