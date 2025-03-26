//! Persistent background jobs using the [`apalis`] crate with a Redis storage backend.

use std::time::Duration;

use anyhow::Context;
use apalis::{
    layers::{retry::RetryPolicy, tracing::MakeSpan, tracing::TraceLayer},
    prelude::*,
};
use apalis_redis::{Config, RedisStorage};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tracing::{Level, Span};

use crate::{
    actions, errors::DomainError, types::DbPool,
    utils::redis_credentials_repo::RedisCredentialsRepo,
};

pub async fn cleanup_sessions(
    credentials_repo: &RedisCredentialsRepo,
    pool: &DbPool,
) {
    let _ = tracing::info!("Running sessions cleanup");
    let mut conn = pool.get().context("Failed to get connection").unwrap();
    let user_ids = actions::users::get_all_user_ids(&mut conn)
        .expect("Failed to get user_ids");
    for user_id in user_ids {
        let _ =
            tracing::info!("Clearing expired sessions for user_id: {user_id}");
        let res = credentials_repo.cleanup_expired_session_ids(&user_id).await;
        if res.is_err() {
            tracing::warn!(
                "Failed to clean expired sessions for user id: {user_id}"
            );
        }
    }
    tokio::time::sleep(Duration::from_secs(10)).await;
}

// Unit struct since we don't need any data, just a trigger for the job
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionsCleanup;

async fn process_sessions_cleanup(
    _job: SessionsCleanup,
    credentials_repo: Data<RedisCredentialsRepo>,
    pool: Data<DbPool>,
) {
    cleanup_sessions(&credentials_repo, &pool).await;
}

pub async fn start_sessions_cleanup_worker(
    credentials_repo: RedisCredentialsRepo,
    pool: DbPool,
) -> anyhow::Result<RedisStorage<SessionsCleanup>> {
    let redis_url = std::env::var("ACTIX_DEMO_REDIS_URL")
        .expect("Missing env variable REDIS_URL");
    let conn = apalis_redis::connect(redis_url).await?;
    let config = Config::default().set_namespace("sessions_cleanup");
    let storage = RedisStorage::new_with_config(conn, config);

    let worker = WorkerBuilder::new("sessions-cleanup")
        .concurrency(1) // Only one cleanup process at a time
        .retry(RetryPolicy::retries(5))
        .layer(TraceLayer::new())
        .data(credentials_repo)
        .data(pool)
        .backend(storage.clone())
        .build_fn(process_sessions_cleanup);

    // // Schedule the cleanup job to run periodically
    // Monitor::new()
    //     .register(worker)
    //     // .repeat_times(std::time::Duration::from_secs(600)) // Run every 10 minutes
    //     .run()
    //     .await?;

    let _ = tokio::spawn(worker.run()).await?;

    Ok(storage)
}

pub async fn start_sessions_cleanup_worker2(
    credentials_repo: RedisCredentialsRepo,
    pool: DbPool,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let policy = backoff::ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_secs(3))
            .with_multiplier(2.0)
            .with_max_interval(Duration::from_secs(30))
            .with_max_elapsed_time(Some(Duration::from_secs(300)))
            .build();

        loop {
            let _ = tracing::info!("Running sessions cleanup");
            let mut conn = match pool.get() {
                Ok(conn) => conn,
                Err(err) => {
                    let _ = tracing::error!("Failed to get connection: {err}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let user_ids = match actions::users::get_all_user_ids(&mut conn) {
                Ok(ids) => ids,
                Err(err) => {
                    let _ = tracing::error!("Failed to get user IDs: {err}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            for user_id in user_ids {
                let operation = || async {
                    let _ = tracing::info!(
                    "Attempting to clear expired sessions for user: {user_id}"
                );

                    credentials_repo
                    .cleanup_expired_session_ids(&user_id)
                    .await
                    .map_err(|err| {
                        backoff::Error::transient(
                            DomainError::new_internal_error(format!(
                                "Session cleanup failed for user: {user_id}: {err}"
                            ))
                        )
                    })
                };

                let retry_result =
                    backoff::future::retry(policy.clone(), operation).await;

                if let Err(err) = retry_result {
                    let _ = tracing::error!(
                    "Permanent failure cleaning sessions for user: {user_id}: {err}"
                );
                }
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    })
}
