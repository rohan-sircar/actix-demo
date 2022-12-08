use std::time::Duration;

use crate::{
    errors::DomainError, models::users::UserId, routes::ws::WsClientFrame,
    utils::RedisChannelReader, AppData,
};

use actix_web::web;
use actix_ws::Session;
use redis::aio::ConnectionManager;

pub async fn msg_receive_loop(
    app_data: web::Data<AppData>,
    user_id: UserId,
    mut session: Session,
) -> Result<(), DomainError> {
    tracing::info!("Starting message channel receive loop ");
    let sub_cm = {
        let client = app_data.redis_conn_factory.clone().ok_or_else(|| {
            DomainError::new_uninitialized_error(
                "redis not initialized".to_owned(),
            )
        })?;
        ConnectionManager::new(client)
            .await
            .map_err(DomainError::from)?
    };

    let mut messages_reader =
        RedisChannelReader::new(format!("messages.{user_id}"), sub_cm, None);

    let mut running = true;
    while running {
        actix_rt::time::sleep(Duration::from_millis(500)).await;
        for msg in messages_reader.get_items().await? {
            let msg = msg.get::<String>("message").unwrap();
            let res = match serde_json::from_str::<WsClientFrame>(&msg) {
                Ok(msg) => {
                    session.text(serde_json::to_string(&msg).unwrap()).await
                }
                Err(err) => {
                    session
                        .text(
                            serde_json::to_string(&WsClientFrame::Error {
                                cause: err.to_string(),
                            })
                            .unwrap(),
                        )
                        .await
                }
            };
            let _ = if res.is_err() {
                running = false;
                break;
            };
            let _ = tracing::debug!("Received message: {:?}", msg);
        }
    }
    Ok(())
}
