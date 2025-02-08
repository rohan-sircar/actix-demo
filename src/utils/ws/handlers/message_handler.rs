use actix_ws::Session;
use redis::{aio::ConnectionManager, AsyncCommands};
use std::{fmt::Display, sync::Arc};
use tracing_futures::Instrument;

use crate::{
    errors::DomainError,
    models::users::UserId,
    models::ws::{SentMessage, WsClientEvent},
    utils, AppData,
};

type RedisPrefixFn = Box<dyn Fn(&dyn Display) -> String + Send + Sync>;

pub async fn handle_send_message(
    _session: Session,
    conn: &mut ConnectionManager,
    user_id: UserId,
    receiver: UserId,
    message: String,
    redis_prefix: &RedisPrefixFn,
) -> Result<(), DomainError> {
    let chan_name = redis_prefix(&format!("messages.{receiver}"));
    let id: String = conn
        .xadd(
            chan_name,
            "*",
            &[(
                "message",
                utils::jstr(&SentMessage {
                    sender: user_id,
                    message,
                }),
            )],
        )
        .await?;
    tracing::info!("Published message with id={id}");
    Ok(())
}

pub async fn process_msg(
    ws_msg: WsClientEvent,
    session: Session,
    conn: &mut ConnectionManager,
    user_id: UserId,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    let redis_prefix = &app_data.redis_prefix;
    match ws_msg {
        WsClientEvent::SendMessage { receiver, message } => {
            handle_send_message(
                session,
                conn,
                user_id,
                receiver,
                message,
                redis_prefix,
            )
            .await
        }
        WsClientEvent::Error { cause } => {
            let _ = tracing::error!("client indicated error {}", cause);
            Ok(())
        }
        WsClientEvent::SubscribeJob { job_id } => {
            let chan_name = redis_prefix(&format!("job.{job_id}"));
            let _ = tracing::info!("Subscribing {chan_name}");
            actix_rt::spawn(
                async move {
                    let res = super::job_handler::subscribe_job(
                        session, chan_name, app_data,
                    )
                    .await;
                    tracing::info!("Job subscription ended: {res:?}");
                }
                .instrument(tracing::info_span!("job_subscribe_loop")),
            );
            Ok(())
        }
    }
}
