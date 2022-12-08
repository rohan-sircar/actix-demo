use std::{sync::Arc, time::Duration};

use crate::{
    errors::DomainError, get_build_info, models::users::UserId,
    utils::RedisChannelReader, AppData,
};
use actix_web::{web, HttpRequest, HttpResponse};

use actix_ws::{Message, MessageStream, Session};
use async_recursion::async_recursion;
use futures::prelude::*;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing_futures::Instrument;

use super::auth::get_claims;

pub async fn build_info_req() -> String {
    serde_json::to_string(get_build_info()).unwrap()
}

// pub struct Context {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WsClientFrame {
    SendMessage { receiver: UserId, message: String },
    Error { cause: String },
}

#[async_recursion(?Send)]
async fn ws_loop(
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

#[derive(Clone, Debug, Deserialize)]
pub struct TokenQuery {
    pub token: Option<String>,
}

#[tracing::instrument(level = "info", skip_all, fields(auth_user_id))]
pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    app_data: web::Data<AppData>,
    token: web::Query<TokenQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let token = token
        .0
        .token
        .ok_or_else(|| DomainError::new_auth_error("need token".to_owned()))?;

    let claims = get_claims(&app_data.jwt_key, &token)?;

    let user_id = claims.custom.user_id;

    let _ = tracing::Span::current().record("auth_user_id", &user_id.as_uint());

    let _ = tracing::info!("Initiating websocket connection");

    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;
    let mut session2 = session.clone();

    let _ = tracing::info!("Websocket connection initiated");

    // let credentials_repo = app_data.credentials_repo.as_ref();
    let app_data2 = app_data.clone();
    let msg_recv_fib = actix_web::rt::spawn(
        async move {
            tracing::info!("Starting message channel receive loop ");
            let sub_cm = {
                let client =
                    app_data2.redis_conn_factory.clone().ok_or_else(|| {
                        DomainError::new_uninitialized_error(
                            "redis not initialized".to_owned(),
                        )
                    })?;
                ConnectionManager::new(client)
                    .await
                    .map_err(DomainError::from)?
            };

            let messages_reader = RedisChannelReader::new(
                format!("messages.{user_id}"),
                sub_cm,
                Arc::new(RwLock::new(None)),
            );

            let mut running = true;
            while running {
                actix_rt::time::sleep(Duration::from_millis(500)).await;
                for msg in messages_reader.get_items().await? {
                    //todo fix the unwrap
                    let msg =
                        msg.get::<String>("message").ok_or_else(|| {
                            DomainError::new_entity_does_not_exist_error(
                                "invalid key".to_owned(),
                            )
                        })?;
                    let res = match serde_json::from_str::<WsClientFrame>(&msg)
                    {
                        Ok(msg) => {
                            session2
                                .text(serde_json::to_string(&msg).unwrap())
                                .await
                        }
                        Err(err) => {
                            session2
                                .text(
                                    serde_json::to_string(
                                        &WsClientFrame::Error {
                                            cause: err.to_string(),
                                        },
                                    )
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
            Ok::<(), DomainError>(())
        }
        .instrument(tracing::info_span!("msg_receive_loop")),
    );

    let mut pub_cm = app_data.redis_conn_manager.clone().ok_or_else(|| {
        DomainError::new_uninitialized_error("redis not initialized".to_owned())
    })?;
    let _ = actix_web::rt::spawn(
        async move {
            let res = ws_loop(session.clone(), msg_stream, &mut pub_cm).await;
            match res {
                Ok(_) => {
                    let _ =
                        tracing::info!("Websocket connection ended successf");
                }
                Err(err) => {
                    let _ = tracing::error!(
                        "Websocket connection ended with error {:?}",
                        err
                    );
                }
            }

            let _ = session.close(None).await;
            let _ = msg_recv_fib.abort();
        }
        .instrument(tracing::info_span!("ws_loop")),
    );

    Ok(response)
}
