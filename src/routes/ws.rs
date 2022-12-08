use crate::{errors::DomainError, models::users::UserId, utils, AppData};
use actix_web::{web, HttpRequest, HttpResponse};

use process_stream::{tokio_stream::StreamExt, Process, ProcessExt};
use serde::{Deserialize, Serialize};
use tracing::debug_span;
use tracing_futures::Instrument;

use super::auth::get_claims;

// pub struct Context {}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WsClientEvent {
    SendMessage { receiver: UserId, message: String },
    RunCommand { args: Vec<String> },
    Error { cause: String },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]

pub enum RunCommandEvent {
    Run { args: Vec<String> },
}

#[derive(Clone, Serialize, Deserialize, Debug)]

pub struct SentMessage {
    pub sender: UserId,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WsServerEvent {
    SentMessage {
        id: String,
        sender: UserId,
        message: String,
    },
    RunCommandMessage {
        line: String,
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
    // let mut session2 = session.clone();

    let _ = tracing::info!("Websocket connection initiated");

    // let credentials_repo = app_data.credentials_repo.as_ref();
    let app_data2 = app_data.clone();
    let session2 = session.clone();
    let msg_recv_fib = actix_web::rt::spawn(
        async move {
            let res =
                utils::ws::msg_receive_loop(app_data2, user_id, session2).await;
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

    let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(50);

    let mut session2 = session.clone();
    let _ = actix_rt::spawn(
        async move {
            while let Some(cmd) = command_rx.recv().await {
                match cmd {
                    RunCommandEvent::Run { args } => {
                        let mut session3 = session2.clone();
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        let _handle = actix_rt::spawn(async move {
                            // let _ = tokio::time::sleep(Duration::from_millis(2500))
                            // .await;
                            let mut ls_home = Process::new("/bin/ls");
                            // let _ = ls_home.arg("-l").arg(".");
                            for arg in args {
                                let _ = ls_home.arg(arg);
                            }
                            let _ = rx.await;
                            let mut stream =
                                ls_home.spawn_and_stream().unwrap();
                            while let Some(output) = stream.next().await {
                                let _ = tracing::debug!("{:?}", output);
                                let line = WsServerEvent::RunCommandMessage {
                                    line: output.to_string(),
                                };
                                let res = session3
                                    .text(serde_json::to_string(&line).unwrap())
                                    .await;
                                let _ = tracing::debug!("{:?}", res);
                            }
                        });
                        if session2.text("started").await.is_err() {
                            break;
                        }
                        tx.send(()).unwrap();
                    }
                }
            }
        }
        .instrument(debug_span!("command_loop")),
    );

    let session2 = session.clone();
    let mut pub_cm = app_data.redis_conn_manager.clone().ok_or_else(|| {
        DomainError::new_uninitialized_error("redis not initialized".to_owned())
    })?;
    let _ = actix_web::rt::spawn(
        async move {
            tracing::info!("Starting websocket loop");
            let res = utils::ws::ws_loop(
                session2,
                msg_stream,
                &mut pub_cm,
                user_id,
                command_tx,
            )
            .await;
            let _ = match res {
                Ok(_) => {
                    let _ =
                        tracing::info!("Websocket connection ended successful");
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
