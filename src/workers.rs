use std::time::Duration;

use tokio::task::JoinHandle;

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
        let policy = backoff::ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_secs(
                config.backoff.initial_interval_secs,
            ))
            .with_multiplier(config.backoff.multiplier)
            .with_max_interval(Duration::from_secs(
                config.backoff.max_interval_secs,
            ))
            .with_max_elapsed_time(Some(Duration::from_secs(
                config.backoff.max_elapsed_time_secs,
            )))
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

            let user_ids = match tokio::task::spawn_blocking(move || {
                actions::users::get_all_user_ids(&mut conn)
            })
            .await
            {
                Ok(Ok(ids)) => ids,
                Ok(Err(err)) => {
                    let _ = tracing::error!("Failed to get user IDs: {err}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                Err(err) => {
                    let _ = tracing::error!(
                        "Failed to execute blocking task: {err}"
                    );
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

            tokio::time::sleep(Duration::from_secs(config.run_interval.into()))
                .await;
        }
    })
}
