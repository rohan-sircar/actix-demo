use std::time::Duration;
use url::Url;

use redis::aio::ConnectionManager;
use reqwest::Client;
use tokio::time::error::Elapsed;

use crate::errors::DomainError;
use crate::types::DbPool;

#[derive(Debug)]
pub enum HealthCheckError {
    Timeout(Elapsed),
    ServiceError(String),
}

impl std::fmt::Display for HealthCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout(e) => write!(f, "Service check timed out: {}", e),
            Self::ServiceError(msg) => write!(f, "{}", msg),
        }
    }
}

impl HealthCheckError {
    pub fn to_domain_error(&self) -> DomainError {
        match self {
            HealthCheckError::Timeout(e) => DomainError::new_internal_error(
                format!("Health check timeout: {e}"),
            ),
            HealthCheckError::ServiceError(msg) => {
                DomainError::new_internal_error(format!(
                    "Health check failed: {msg}"
                ))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresHealthChecker {
    pool: DbPool,
}

impl PostgresHealthChecker {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn check_health(
        &self,
        timeout: Duration,
    ) -> Result<(), HealthCheckError> {
        use diesel::prelude::*;
        let pool = self.pool.clone();
        tokio::time::timeout(timeout, async move {
            let mut conn = pool.get().map_err(|e| {
                HealthCheckError::ServiceError(format!(
                    "Failed to get DB connection: {e}"
                ))
            })?;

            tokio::task::spawn_blocking(move || {
                diesel::sql_query("SELECT 1")
                    .execute(&mut conn)
                    .map_err(|e| {
                        HealthCheckError::ServiceError(format!(
                            "DB query failed: {e}"
                        ))
                    })
            })
            .await
            .map_err(|e| {
                HealthCheckError::ServiceError(format!("Task join error: {e}"))
            })??;

            Ok(())
        })
        .await
        .map_err(HealthCheckError::Timeout)?
    }
}

#[derive(Clone)]
pub struct RedisHealthChecker {
    conn_manager: ConnectionManager,
}

impl RedisHealthChecker {
    pub fn new(conn_manager: ConnectionManager) -> Self {
        Self { conn_manager }
    }

    pub async fn check_health(
        &self,
        timeout: Duration,
    ) -> Result<(), HealthCheckError> {
        tokio::time::timeout(timeout, async move {
            let mut conn = self.conn_manager.clone();
            let () = redis::cmd("PING").query_async(&mut conn).await.map_err(
                |e| {
                    HealthCheckError::ServiceError(format!(
                        "Redis ping failed: {e}"
                    ))
                },
            )?;
            Ok(())
        })
        .await
        .map_err(HealthCheckError::Timeout)?
    }
}

#[derive(Clone)]
pub struct UrlHealthChecker {
    client: Client,
    endpoint: Url,
    service_name: String,
}

impl UrlHealthChecker {
    pub fn new(client: Client, endpoint: Url, service_name: String) -> Self {
        Self {
            client,
            endpoint,
            service_name,
        }
    }

    pub async fn check_health(
        &self,
        timeout: Duration,
    ) -> Result<(), HealthCheckError> {
        let client = self.client.clone();
        tokio::time::timeout(timeout, async move {
            let response =
                client.get(self.endpoint.clone()).send().await.map_err(
                    |e| {
                        HealthCheckError::ServiceError(format!(
                            "Failed to send request to {}: {e}",
                            self.service_name
                        ))
                    },
                )?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(HealthCheckError::ServiceError(format!(
                    "{} health check failed with status: {}",
                    self.service_name,
                    response.status()
                )))
            }
        })
        .await
        .map_err(HealthCheckError::Timeout)?
    }
}

pub enum HealthChecker {
    Postgres(PostgresHealthChecker),
    Redis(RedisHealthChecker),
    Loki(UrlHealthChecker),
    Prometheus(UrlHealthChecker),
}

impl HealthChecker {
    pub async fn check_health(
        &self,
        timeout: Duration,
    ) -> Result<(), HealthCheckError> {
        match self {
            HealthChecker::Postgres(checker) => {
                checker.check_health(timeout).await
            }
            HealthChecker::Redis(checker) => {
                checker.check_health(timeout).await
            }
            HealthChecker::Loki(checker) => checker.check_health(timeout).await,
            HealthChecker::Prometheus(checker) => {
                checker.check_health(timeout).await
            }
        }
    }
}

pub type HealthcheckName = &'static str;

pub fn create_health_checkers(
    pool: DbPool,
    conn_manager: ConnectionManager,
    loki_endpoint: url::Url,
    prometheus_endpoint: url::Url,
    client: Client,
) -> Vec<(HealthcheckName, HealthChecker)> {
    let loki_hc = loki_endpoint
        .join("/ready")
        .expect("Expect valid loki endpoint");
    let prometheus_hc = prometheus_endpoint
        .join("/-/healthy")
        .expect("Expect valid prometheus endpoint");
    vec![
        (
            "postgresql",
            HealthChecker::Postgres(PostgresHealthChecker::new(pool)),
        ),
        (
            "redis",
            HealthChecker::Redis(RedisHealthChecker::new(conn_manager)),
        ),
        (
            "loki",
            HealthChecker::Loki(UrlHealthChecker::new(
                client.clone(),
                loki_hc,
                "Loki".to_owned(),
            )),
        ),
        (
            "prometheus",
            HealthChecker::Prometheus(UrlHealthChecker::new(
                client,
                prometheus_hc,
                "Prometheus".to_owned(),
            )),
        ),
    ]
}
