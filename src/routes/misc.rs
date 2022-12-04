use crate::{errors::DomainError, get_build_info, AppData};
use actix_web::{
    error::ErrorInternalServerError, web, HttpRequest, HttpResponse,
};

use actix_ws::{Message, MessageStream, Session};
use async_recursion::async_recursion;
use futures::StreamExt;
use redis::{aio::Connection, AsyncCommands};
use serde::Deserialize;

use super::auth::get_claims;

pub async fn build_info_req() -> String {
    serde_json::to_string(get_build_info()).unwrap()
}

// pub struct Context {}

#[async_recursion(?Send)]
async fn ws_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
    // mut conn: PubSub,
    mut conn: Connection,
    channel_name: String,
    _app_data: &AppData,
) -> Result<(), DomainError> {
    match msg_stream.next().await {
        Some(Ok(msg)) => match msg {
            Message::Ping(bytes) => {
                if session.pong(&bytes).await.is_ok() {
                    ws_loop(session, msg_stream, conn, channel_name, _app_data)
                        .await
                } else {
                    Ok(())
                }
            }
            Message::Text(s) => {
                tracing::debug!("Got text, {}", s);
                let _ = conn
                    .publish::<&str, String, ()>(
                        &channel_name,
                        format!("Redis publishing message {}", s),
                    )
                    .await?;
                if session.text(s.to_string()).await.is_ok() {
                    // let _ = app_data
                    //     .credentials_repo
                    //     .load(&UserId::from_str("1").unwrap())
                    //     .await;
                    ws_loop(session, msg_stream, conn, channel_name, _app_data)
                        .await
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
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;
    let mut session2 = session.clone();
    // let credentials_repo = app_data.credentials_repo.as_ref();

    // let mb_request_id = req.headers().get("request_id").cloned();

    // let _ = tracing::info!("request id = {:?}", mb_request_id);

    let r_pub = app_data
        .redis_conn_factory
        .as_ref()
        .ok_or_else(|| ErrorInternalServerError("redis not initialized"))?
        .get_async_connection()
        .await
        .map_err(DomainError::from)?;

    let claims = get_claims(&app_data.jwt_key, &token)?;

    let user_id = claims.custom.user_id;

    let channel_name1 = move || {
        let target_user_id = if user_id.as_uint() == 1 { 2 } else { 1 };
        format!("messages.{target_user_id}")
    };

    let channel_name2 = move || {
        let target_user_id = if user_id.as_uint() == 1 { 1 } else { 2 };
        format!("messages.{target_user_id}")
    };

    let app_data2 = app_data.clone();

    let _ = tokio::spawn(async move {
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
        let _ = r_sub.subscribe(channel_name2()).await?;
        let mut r_stream = r_sub.on_message();

        let _ = while let Some(msg) = r_stream.next().await {
            let msg = msg.get_payload::<String>()?;
            //todo fix this
            let _ = if session2.text(&msg).await.is_err() {
                break;
            };
            tracing::info!("Redis pubsub received message: {:?}", msg);
        };
        tracing::info!("Redis stream ended");
        Ok::<(), DomainError>(())
    });

    let _ = actix_rt::spawn(async move {
        let res = ws_loop(
            session.clone(),
            msg_stream,
            r_pub,
            channel_name1(),
            &app_data.clone(),
        )
        .await;
        match res {
            Ok(_) => {
                let _ =
                    tracing::info!("Websocket connection ended successfully");
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
    });

    Ok(response)
}
