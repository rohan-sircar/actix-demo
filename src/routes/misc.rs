use crate::{
    errors::DomainError, get_build_info, models::users::UserId, AppData,
};
use actix_web::{
    error::ErrorInternalServerError, web, HttpRequest, HttpResponse,
};

use actix_ws::{Message, MessageStream, Session};
use async_recursion::async_recursion;
use futures::StreamExt;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};

use super::auth::get_claims;

pub async fn build_info_req() -> String {
    serde_json::to_string(get_build_info()).unwrap()
}

// pub struct Context {}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientFrame {
    SendMessage { receiver: UserId, message: String },
}

#[async_recursion(?Send)]
async fn ws_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
    conn: &mut ConnectionManager,
    _app_data: &AppData,
) -> Result<(), DomainError> {
    match msg_stream.next().await {
        Some(Ok(msg)) => match msg {
            Message::Ping(bytes) => {
                if session.pong(&bytes).await.is_ok() {
                    ws_loop(session, msg_stream, conn, _app_data).await
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
                            let _ = conn
                                .publish(
                                    format!("messages.{}", &receiver),
                                    s.to_string(),
                                )
                                .await?;
                            Ok(())
                        }
                    },
                    Err(err) => session.text(format!("Error: {:?}", err)).await,
                };

                if res.is_ok() {
                    ws_loop(session, msg_stream, conn, _app_data).await
                } else {
                    Ok(())
                }
            }
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

#[tracing::instrument(level = "info", skip(app_data, req, body, token))]
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

    let _ =
        tracing::info!("Initiating websocket connection for user_id={user_id}");

    let app_data2 = app_data.clone();

    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;
    let mut session2 = session.clone();

    let _ =
        tracing::info!("Websocket connection initiated for user_id={user_id}");

    // let credentials_repo = app_data.credentials_repo.as_ref();

    // let mb_request_id = req.headers().get("request_id").cloned();

    // let _ = tracing::info!("request id = {:?}", mb_request_id);

    let recv_stream_handle = tokio::spawn(async move {
        let mut r_sub = app_data2
            .redis_conn_factory
            .as_ref()
            .ok_or_else(|| {
                DomainError::new_uninitialized_error(
                    "redis not initialized".to_owned(),
                )
            })?
            .get_async_connection()
            .await?
            .into_pubsub();
        let _ = r_sub.subscribe(format!("messages.{user_id}")).await?;
        {
            let mut r_stream = r_sub.on_message();

            let _ = while let Some(msg) = r_stream.next().await {
                let msg = msg.get_payload::<String>()?;
                let res = session2.text(&msg).await;
                let _ = if res.is_err() {
                    break;
                };

                tracing::debug!("Redis pubsub received message: {:?}", msg);
            };
        }
        let _ = r_sub.unsubscribe(format!("messages.{user_id}")).await?;
        tracing::info!("Redis stream ended");
        Ok::<(), DomainError>(())
    });

    let r_pub = app_data
        .redis_conn_manager
        .as_ref()
        .ok_or_else(|| ErrorInternalServerError("redis not initialized"))?
        .clone();

    let _ = actix_rt::spawn(async move {
        let res = ws_loop(
            session.clone(),
            msg_stream,
            &mut r_pub.clone(),
            &app_data.clone(),
        )
        .await;
        match res {
            Ok(_) => {
                let _ = tracing::info!(
                    "Websocket connection ended successfully user_id={user_id}"
                );
            }
            Err(err) => {
                let _ = tracing::error!(
                    "Websocket connection ended with error {:?}",
                    err
                );
            }
        }

        // let _ = app_data
        //     .credentials_repo
        //     .load(&UserId::from_str("1").unwrap());

        let _ = session.close(None).await;
        let _ = recv_stream_handle.abort();
    });

    Ok(response)
}
