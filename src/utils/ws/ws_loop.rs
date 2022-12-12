use crate::{
    errors::DomainError,
    models::users::UserId,
    routes::ws::{RunCommandEvent, SentMessage, WsClientEvent, WsServerEvent},
};

use actix_ws::{Message, MessageStream, Session};
use async_recursion::async_recursion;
use futures::prelude::*;
use redis::{aio::ConnectionManager, AsyncCommands};
use tokio::sync::mpsc::Sender;

// pub enum WsResult {
//     Ok,
//     Closed,
// }

#[async_recursion(?Send)]
pub async fn ws_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
    conn: &mut ConnectionManager,
    user_id: UserId,
    command_tx: Sender<RunCommandEvent>,
) -> Result<(), DomainError> {
    match msg_stream.next().await {
        Some(Ok(msg)) => match msg {
            Message::Ping(bytes) => {
                if session.pong(&bytes).await.is_ok() {
                    ws_loop(session, msg_stream, conn, user_id, command_tx)
                        .await
                } else {
                    Ok(())
                }
            }
            Message::Text(s) => {
                tracing::debug!("Got text, {}", s);
                let res = match serde_json::from_str::<WsClientEvent>(&s) {
                    Ok(ws_msg) => {
                        Ok(process_msg(ws_msg, conn, user_id, &command_tx)
                            .await?)
                    }
                    Err(err) => {
                        let err = &WsServerEvent::Error {
                            id: None,
                            cause: err.to_string(),
                        };
                        let err = serde_json::to_string(err).unwrap();
                        session.text(err).await
                    }
                };

                if res.is_ok() {
                    ws_loop(session, msg_stream, conn, user_id, command_tx)
                        .await
                } else {
                    Ok(())
                }
            }
            Message::Close(_) => Ok(()),
            _ => Ok(()),
        },
        Some(Err(err)) => Err(DomainError::from(err)),
        None => Ok(()),
    }
}

pub async fn process_msg(
    ws_msg: WsClientEvent,
    // mut session: Session,
    conn: &mut ConnectionManager,
    user_id: UserId,
    command_tx: &Sender<RunCommandEvent>,
) -> Result<(), DomainError> {
    match ws_msg {
        WsClientEvent::SendMessage { receiver, message } => {
            let chan_name = format!("messages.{receiver}");
            let id: String = conn
                .xadd(
                    chan_name,
                    "*",
                    &[(
                        "message",
                        serde_json::to_string(&SentMessage {
                            sender: user_id,
                            message: message.clone(),
                        })
                        .unwrap(),
                    )],
                )
                .await?;
            tracing::info!("Published message with id={id}");
            Ok(())
        }
        WsClientEvent::Error { cause } => {
            let _ = tracing::error!("client indicated error {}", cause);
            Ok(())
        }
        WsClientEvent::RunCommand { args } => {
            let msg = RunCommandEvent::Run { args };
            let _ = command_tx.send(msg).await.unwrap();
            Ok(())
        }
    }
}
