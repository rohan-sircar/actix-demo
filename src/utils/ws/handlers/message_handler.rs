use actix_ws::Session;
use redis::{aio::ConnectionManager, AsyncCommands};

use crate::{
    errors::DomainError,
    models::{users::UserId, ws::SentMessage},
    types::RedisPrefixFn,
    utils,
};

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
