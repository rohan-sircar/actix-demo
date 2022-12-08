use std::time::Duration;

use crate::{
    errors::DomainError,
    models::users::UserId,
    routes::ws::{SentMessage, WsServerEvent},
    utils::{RedisChannelReader, RedisReply},
    AppData,
};

use actix_web::web;
use actix_ws::Session;
use redis::aio::ConnectionManager;

pub async fn msg_receive_loop(
    app_data: web::Data<AppData>,
    user_id: UserId,
    mut session: Session,
) -> Result<(), DomainError> {
    let _ = tracing::info!("Starting message channel receive loop ");
    let cm = {
        let client = app_data.redis_conn_factory.clone().ok_or_else(|| {
            DomainError::new_uninitialized_error(
                "redis not initialized".to_owned(),
            )
        })?;
        ConnectionManager::new(client)
            .await
            .map_err(DomainError::from)?
    };

    let mut messages_reader = RedisChannelReader::<SentMessage>::new(
        format!("messages.{user_id}"),
        cm,
        None,
    );

    let mut running = true;
    while running {
        actix_rt::time::sleep(Duration::from_millis(500)).await;
        for msg in messages_reader.get_items2().await? {
            let _ = tracing::debug!("Received message: {:?}", &msg);
            let res = match msg {
                RedisReply::Success { id, data } => {
                    let msg = WsServerEvent::SentMessage {
                        id,
                        sender: data.sender,
                        message: data.message,
                    };
                    session.text(serde_json::to_string(&msg).unwrap()).await
                }
                RedisReply::Error { id, cause } => {
                    let msg = WsServerEvent::Error {
                        id: Some(id),
                        cause,
                    };
                    session.text(serde_json::to_string(&msg).unwrap()).await
                }
            };
            let _ = if res.is_err() {
                running = false;
                break;
            };
        }
    }
    Ok(())
}
