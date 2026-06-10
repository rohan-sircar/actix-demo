use actix_ws::Session;
use redis::{aio::ConnectionManager, AsyncCommands};

use crate::{
    errors::DomainError,
    models::{users::UserUuid, ws::SentMessage},
    types::RedisPrefixFn,
    utils,
};

pub async fn handle_send_message(
    _session: Session,
    conn: &mut ConnectionManager,
    user_uuid: UserUuid,
    receiver: UserUuid,
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
                    sender: user_uuid,
                    message,
                }),
            )],
        )
        .await?;
    tracing::info!("Published message with id={id}");
    Ok(())
}
