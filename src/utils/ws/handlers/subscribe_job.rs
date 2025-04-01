use actix_ws::Session;
use futures::StreamExt;
use std::sync::Arc;

use crate::{
    actions,
    errors::DomainError,
    models::ws::{MyProcessItem, WsServerEvent},
    utils::{self, ws::SessionExt},
    AppData,
};
use actix_web::web;

pub async fn handle_subscribe_job(
    mut session: Session,
    unverified_job_id: uuid::Uuid,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    let _ = tracing::info!("Verifying job exists before subscribing...");
    let pool = app_data.pool.clone();
    let mb_job = web::block(move || {
        let mut conn = pool.get()?;
        actions::misc::get_job_by_uuid(unverified_job_id, &mut conn)
    })
    .await??;

    let job = match mb_job {
        Some(job) => {
            let _ = tracing::info!("Job with id: {unverified_job_id} exists.");
            Ok(job)
        }
        None => {
            let err = DomainError::new_entity_does_not_exist_error(format!(
                "Job with id: {unverified_job_id} does not exist"
            ));
            session
                .send_server_event(WsServerEvent::Error {
                    id: None,
                    cause: format!(
                        "Job with id: {unverified_job_id} does not exist"
                    ),
                })
                .await?;
            Err(err)
        }
    }?;

    let redis_prefix = &app_data.redis_prefix;
    let job_id = job.job_id;
    let chan_name = redis_prefix(&format!("job.{job_id}"));
    let _ = tracing::info!("Subscribing to Redis channel {chan_name}...");
    let mut ps = app_data
        .redis_conn_factory
        .clone()
        .get_async_pubsub()
        .await?;
    let _ = ps.subscribe(&chan_name).await?;
    let _ =
        tracing::info!("Successfully subscribed to Redis channel {chan_name}.");

    {
        let mut msg_stream = ps.on_message();
        while let Some(msg) = msg_stream.next().await {
            let cmd = msg.get_payload::<String>().unwrap_or_default();
            let _ = tracing::debug!("Received command message: {cmd}");
            let rcm = match serde_json::from_str::<MyProcessItem>(&cmd) {
                Ok(rcm) => rcm,
                Err(_) => {
                    tracing::error!("Failed to parse command: {cmd}");
                    continue;
                }
            };
            let server_msg = WsServerEvent::CommandMessage {
                message: rcm.clone(),
            };

            let msg_str = utils::jstr(&server_msg);

            let should_break = match &rcm {
                MyProcessItem::Line { .. } | MyProcessItem::Error { .. } => {
                    let _ = session.text(msg_str.clone()).await;
                    let _ =
                        tracing::debug!("Sent message to client: {msg_str}");
                    session.text(msg_str).await.is_err()
                }
                MyProcessItem::Done { code } => {
                    let _ =
                        tracing::info!("Process completed with code={code}");
                    let _ = session.text(msg_str).await;
                    true
                }
            };

            if should_break {
                let _ = tracing::info!(
                    "Break received, stopping message processing."
                );
                break;
            }
        }
    }

    let _ = tracing::info!("Unsubscribing from Redis channel {chan_name}...");
    ps.unsubscribe(&chan_name).await?;
    let _ = tracing::info!(
        "Successfully unsubscribed from Redis channel {chan_name}."
    );
    Ok(())
}
