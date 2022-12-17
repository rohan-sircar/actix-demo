use crate::{errors::DomainError, models::users::UserId, utils, AppData};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::{Deserialize, Serialize};
use tracing_futures::Instrument;

use super::{auth::get_claims, command::MyProcessItem};

// pub struct Context {}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]

pub enum WsClientEvent {
    SendMessage {
        receiver: UserId,
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    SubscribeJob {
        job_id: String,
    },
    Error {
        cause: String,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]

pub struct SentMessage {
    pub sender: UserId,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum WsServerEvent {
    SentMessage {
        id: String,
        sender: UserId,
        message: String,
    },
    CommandMessage {
        message: MyProcessItem,
    },
    Error {
        id: Option<String>,
        cause: String,
    },
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

    let _ = tracing::info!("Websocket connection initiated");

    // let credentials_repo = app_data.credentials_repo.as_ref();
    let session2 = session.clone();
    let cm = utils::get_redis_conn(app_data.clone().into_inner()).await?;

    let msg_recv_fib = actix_web::rt::spawn(
        async move {
            let res = utils::ws::msg_receive_loop(user_id, cm, session2).await;
            let _ = match res {
                Ok(_) => {
                    let _ = tracing::info!("Msg receive loop ended successful");
                }
                Err(err) => {
                    let _ = tracing::error!(
                        "Msg receive loop ended with error {:?}",
                        err
                    );
                }
            };
        }
        .instrument(tracing::info_span!("msg_receive_loop")),
    );

    let session2 = session.clone();
    let mut pub_cm =
        utils::get_redis_conn(app_data.clone().into_inner()).await?;
    let _ = actix_web::rt::spawn(
        async move {
            tracing::info!("Starting websocket loop");
            let res = utils::ws::ws_loop(
                session2,
                msg_stream,
                &mut pub_cm,
                user_id,
                app_data.into_inner().clone(),
            )
            .await;
            let _ = match res {
                Ok(_) => {
                    let _ = tracing::info!(
                        "Websocket connection ended successfully"
                    );
                }
                Err(err) => {
                    let _ = tracing::error!(
                        "Websocket connection ended with error {:?}",
                        err
                    );
                }
            };

            let _ = session.close(None).await;
            let _ = msg_recv_fib.abort();
        }
        .instrument(tracing::info_span!("ws_loop")),
    );

    Ok(response)
}
