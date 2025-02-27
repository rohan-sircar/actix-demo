use std::sync::Arc;

use crate::{
    errors::DomainError, models::users::UserId, models::ws::WsClientEvent,
    models::ws::WsServerEvent, utils, AppData,
};

use utils::ws;

use actix_ws::{Message, MessageStream, Session};
use futures::StreamExt;
use redis::aio::ConnectionManager;

pub async fn ws_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
    conn: &mut ConnectionManager,
    user_id: UserId,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    while let Some(item) = msg_stream.next().await {
        match item {
            Ok(Message::Ping(bytes)) => {
                if session.pong(&bytes).await.is_err() {
                    break;
                }
            }
            Ok(Message::Text(s)) => {
                tracing::info!("Received message");
                tracing::debug!("Message content: {}", s);
                let res = match serde_json::from_str::<WsClientEvent>(&s) {
                    Ok(ws_msg) => Ok(ws::process_client_msg(
                        ws_msg,
                        session.clone(),
                        conn,
                        user_id,
                        app_data.clone(),
                    )
                    .await?),
                    Err(err) => {
                        let err = &WsServerEvent::Error {
                            id: None,
                            cause: err.to_string(),
                        };
                        let err = utils::jstr(err);
                        session.text(err).await
                    }
                };

                if res.is_err() {
                    break;
                }
            }
            Ok(Message::Close(reason)) => {
                tracing::info!("Received close, reason={:?}", reason);
                break;
            }
            Ok(_) => {
                break;
            }
            Err(_) => {
                break;
            }
        }
    }
    Ok(())
}
