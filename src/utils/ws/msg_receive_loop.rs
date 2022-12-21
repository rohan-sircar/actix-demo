use std::{sync::Arc, time::Duration};

use crate::{
    errors::DomainError,
    models::users::UserId,
    routes::ws::{SentMessage, WsServerEvent},
    utils::{self, RedisChannelReader, RedisReply},
    AppData,
};

use actix_ws::Session;
use redis::{aio::ConnectionManager, streams::StreamReadOptions};

pub async fn msg_receive_loop(
    user_id: UserId,
    cm: ConnectionManager,
    mut session: Session,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    let _ = tracing::info!("Starting message channel receive loop ");

    let opts = StreamReadOptions::default().block(500).count(5);

    let redis_prefix = app_data.redis_prefix.as_ref();

    let mut messages_reader = RedisChannelReader::<SentMessage>::new(
        redis_prefix(&format!("messages.{user_id}")),
        cm,
        None,
        opts,
    );

    let mut running = true;
    while running {
        actix_rt::time::sleep(Duration::from_millis(500)).await;
        let msgs = messages_reader.get_items().await?;
        let len = msgs.len();
        if len > 0 {
            tracing::info!("Received {} messages", len);
        }
        for msg in msgs {
            let _ = tracing::debug!("Received message: {:?}", &msg);
            let msg = match msg {
                RedisReply::Ok { id, data } => WsServerEvent::SentMessage {
                    id,
                    sender: data.sender,
                    message: data.message,
                },
                RedisReply::Error { id, cause } => WsServerEvent::Error {
                    id: Some(id),
                    cause,
                },
            };
            let res = session.text(utils::jstr(&msg)).await;
            let _ = if res.is_err() {
                running = false;
                break;
            };
        }
    }
    Ok(())
}
