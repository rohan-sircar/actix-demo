use crate::{errors::DomainError, routes::ws::WsClientFrame};

use actix_ws::{Message, MessageStream, Session};
use async_recursion::async_recursion;
use futures::prelude::*;
use redis::{aio::ConnectionManager, AsyncCommands};

#[async_recursion(?Send)]
pub async fn ws_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
    conn: &mut ConnectionManager,
) -> Result<(), DomainError> {
    match msg_stream.next().await {
        Some(Ok(msg)) => match msg {
            Message::Ping(bytes) => {
                if session.pong(&bytes).await.is_ok() {
                    ws_loop(session, msg_stream, conn).await
                } else {
                    Ok(())
                }
            }
            Message::Text(s) => {
                tracing::debug!("Got text, {}", s);
                let res = match serde_json::from_str::<WsClientFrame>(&s) {
                    Ok(ws_msg) => match ws_msg {
                        WsClientFrame::SendMessage {
                            receiver,
                            message: _,
                        } => {
                            let id: String = conn
                                .xadd(
                                    format!("messages.{receiver}"),
                                    "*",
                                    &[("message", s.to_string())],
                                )
                                .await?;
                            tracing::debug!("Message id was {id}");
                            Ok(())
                        }
                        WsClientFrame::Error { cause: _ } => Ok(()),
                    },
                    Err(err) => session.text(format!("Error: {:?}", err)).await,
                };

                if res.is_ok() {
                    ws_loop(session, msg_stream, conn).await
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
