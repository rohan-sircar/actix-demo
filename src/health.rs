use std::time::Duration;

use redis::aio::ConnectionManager;
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

pub trait HealthCheckable {
    fn check_health(
        &self,
        timeout: Duration,
    ) -> impl std::future::Future<Output = Result<(), HealthCheckError>> + Send;
}

#[derive(Debug, Clone)]
pub struct PostgresHealthChecker {
    pool: DbPool,
}

impl PostgresHealthChecker {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl HealthCheckable for PostgresHealthChecker {
    async fn check_health(
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
        .map_err(|e| HealthCheckError::Timeout(e))?
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
}

impl HealthCheckable for RedisHealthChecker {
    async fn check_health(
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
        .map_err(|e| HealthCheckError::Timeout(e))?
    }
}
pub enum HealthChecker {
    Postgres(PostgresHealthChecker),
    Redis(RedisHealthChecker),
}

impl HealthCheckable for HealthChecker {
    async fn check_health(
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
        }
    }
}

pub struct HealthCheckers {
    pub postgres: PostgresHealthChecker,
    pub redis: RedisHealthChecker,
}

impl HealthCheckers {
    pub fn new(pool: DbPool, conn_manager: ConnectionManager) -> Self {
        Self {
            postgres: PostgresHealthChecker::new(pool),
            redis: RedisHealthChecker::new(conn_manager),
        }
    }

    pub fn get_checkers(&self) -> Vec<(&'static str, HealthChecker)> {
        vec![
            ("postgresql", HealthChecker::Postgres(self.postgres.clone())),
            ("redis", HealthChecker::Redis(self.redis.clone())),
        ]
    }
}
