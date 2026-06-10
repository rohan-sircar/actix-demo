use std::time::Duration;

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
        // TODO backoff is unmaintained, use a different crate
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
                let operation = || async {
                    let _ = tracing::debug!(
                    "Attempting to clear expired sessions for user: {user_uuid}"
                );

                    credentials_repo
                    .cleanup_expired_session_ids(&user_uuid)
                    .await
                    .map_err(|err| {
                        backoff::Error::transient(
                            DomainError::new_internal_error(format!(
                                "Session cleanup failed for user: {user_uuid}: {err}"
                            ))
                        )
                    })
                };

                let retry_result =
                    backoff::future::retry(policy.clone(), operation).await;

                if let Err(err) = retry_result {
                    let _ = tracing::error!(
                    "Permanent failure cleaning sessions for user: {user_uuid}: {err}"
                );
                }
            }

            sleep(Duration::from_secs(config.run_interval.into())).await;
        }
    })
}
