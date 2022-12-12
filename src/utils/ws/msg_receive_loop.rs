use std::time::Duration;

use crate::{
    errors::DomainError,
    models::users::UserId,
    routes::ws::{SentMessage, WsServerEvent},
    utils::{RedisChannelReader, RedisReply},
};

use actix_ws::Session;
use redis::{aio::ConnectionManager, streams::StreamReadOptions};

pub async fn msg_receive_loop(
    user_id: UserId,
    cm: ConnectionManager,
    mut session: Session,
) -> Result<(), DomainError> {
    let _ = tracing::info!("Starting message channel receive loop ");

    let opts = StreamReadOptions::default().block(500).count(5);

    let mut messages_reader = RedisChannelReader::<SentMessage>::new(
        format!("messages.{user_id}"),
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
            let res = session.text(serde_json::to_string(&msg).unwrap()).await;
            let _ = if res.is_err() {
                running = false;
                break;
            };
        }
    }
    Ok(())
}
