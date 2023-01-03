use std::sync::Arc;

use crate::{
    errors::DomainError,
    models::users::UserId,
    models::ws::MyProcessItem,
    models::ws::{SentMessage, WsClientEvent, WsServerEvent},
    utils, AppData,
};

use actix_ws::{Message, MessageStream, Session};
use futures::prelude::*;
use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::{info_span, Instrument};

// pub enum WsResult {
//     Ok,
//     Closed,
// }

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
                    Ok(ws_msg) => Ok(process_msg(
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

pub async fn process_msg(
    ws_msg: WsClientEvent,
    session: Session,
    conn: &mut ConnectionManager,
    user_id: UserId,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    let redis_prefix = app_data.redis_prefix.as_ref();
    match ws_msg {
        WsClientEvent::SendMessage { receiver, message } => {
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
        WsClientEvent::Error { cause } => {
            let _ = tracing::error!("client indicated error {}", cause);
            Ok(())
        }
        WsClientEvent::SubscribeJob { job_id } => {
            let chan_name = redis_prefix(&format!("job.{job_id}"));
            let _ = tracing::info!("Subscribing {chan_name}");
            let _ = tokio::spawn(
                async move {
                    let res = async move {
                        let mut session2 = session.clone();
                        let mut ps = utils::get_pubsub(app_data).await?;
                        let _ = ps.subscribe(&chan_name).await?;
                        {
                            let mut msg_stream = ps.on_message();
                            while let Some(msg) = msg_stream.next().await {
                                let cmd = msg
                                    .get_payload::<String>()
                                    .unwrap_or_default();
                                let _ = tracing::debug!("Got cmd {cmd}");
                                let rcm =
                                    serde_json::from_str::<MyProcessItem>(&cmd)
                                        .unwrap();
                                let server_msg =
                                    WsServerEvent::CommandMessage {
                                        message: rcm,
                                    };

                                let msg_str = utils::jstr(&server_msg);

                                let _ = match &server_msg {
                                    WsServerEvent::CommandMessage {
                                        message,
                                    } => match message {
                                        MyProcessItem::Line { value: _ } => {
                                            let res =
                                                session2.text(msg_str).await;
                                            if res.is_err() {
                                                break;
                                            }
                                        }
                                        MyProcessItem::Error { cause: _ } => {
                                            let res =
                                                session2.text(msg_str).await;
                                            if res.is_err() {
                                                break;
                                            }
                                        }
                                        MyProcessItem::Done { code } => {
                                            let _ = tracing::info!(
                                            "Process completed with code={code}"
                                        );
                                            let _ =
                                                session2.text(msg_str).await;
                                            break;
                                        }
                                    },
                                    _ => panic!("Coding error lol"),
                                };
                            }
                        }
                        ps.unsubscribe(&chan_name).await?;
                        // //not sure if this is required
                        // drop(ps);
                        Ok::<(), DomainError>(())
                    }
                    .await;
                    tracing::info!("res = {:?}", res);
                }
                .instrument(info_span!("command_receive_loop")),
            );
            Ok(())
        }
    }
}
