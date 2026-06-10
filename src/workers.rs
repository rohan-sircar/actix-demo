use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use backon::{ExponentialBuilder, Retryable};
use tokio::{task::JoinHandle, time::sleep};

use crate::{
    actions, errors::DomainError, models::worker::WorkerConfig, types::DbPool,
    utils::redis_credentials_repo::RedisCredentialsRepo,
};

pub async fn start_sessions_cleanup_worker(
    config: WorkerConfig,
    credentials_repo: RedisCredentialsRepo,
    pool: DbPool,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let builder = ExponentialBuilder::new()
            .with_min_delay(Duration::from_secs(
                config.backoff.initial_interval_secs,
            ))
            .with_factor(config.backoff.multiplier as f32)
            .with_max_delay(Duration::from_secs(
                config.backoff.max_interval_secs,
            ))
            .with_total_delay(Some(Duration::from_secs(
                config.backoff.max_elapsed_time_secs,
            )));

        loop {
            let _ = tracing::debug!("Running sessions cleanup");
            let mut conn = match pool.get() {
                Ok(conn) => conn,
                Err(err) => {
                    let _ = tracing::error!("Failed to get connection: {err}");
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let user_uuids = match tokio::task::spawn_blocking(move || {
                actions::users::get_all_user_uuids(&mut conn)
            })
            .await
            {
                Ok(Ok(ids)) => ids,
                Ok(Err(err)) => {
                    let _ = tracing::error!("Failed to get user UUIDs: {err}");
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
                Err(err) => {
                    let _ = tracing::error!(
                        "Failed to execute blocking task: {err}"
                    );
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            for user_uuid in user_uuids {
                let operation = || {
                    let credentials_repo = &credentials_repo;
                    async move {
                        let _ = tracing::debug!(
                        "Attempting to clear expired sessions for user: {user_uuid}"
                    );

                        credentials_repo
                        .cleanup_expired_session_ids(&user_uuid)
                        .await
                        .map_err(|err| {
                            DomainError::new_internal_error(format!(
                                "Session cleanup failed for user: {user_uuid}: {err}"
                            ))
                        })
                    }
                };

                let attempt_count = AtomicU32::new(1);
                let result = operation
                    .retry(builder)
                    .notify(|err, dur| {
                        let attempt = attempt_count.fetch_add(1, Ordering::Relaxed) + 1;
                        tracing::warn!(
                            "Retry {attempt} for {user_uuid}: {err} (backoff {dur:?})",
                        );
                    })
                    .when(|e| e.is_transient())
                    .await;

                if let Err(err) = result {
                    let total = attempt_count.load(Ordering::Relaxed);
                    let _ = tracing::error!(
                        "Failed {user_uuid} after {total} attempts: {err}",
                    );
                }
            }

            sleep(Duration::from_secs(config.run_interval.into())).await;
        }
    })
}
