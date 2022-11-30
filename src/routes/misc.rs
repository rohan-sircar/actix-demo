use std::str::FromStr;

use crate::{get_build_info, models::users::UserId, AppData};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::{Message, MessageStream, Session};
use async_recursion::async_recursion;
use futures::StreamExt;

pub async fn build_info_req() -> String {
    serde_json::to_string(get_build_info()).unwrap()
}

#[async_recursion(?Send)]
async fn ws_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
    app_data: &AppData,
) -> Result<(), String> {
    match msg_stream.next().await {
        Some(Ok(msg)) => match msg {
            Message::Ping(bytes) => {
                if session.pong(&bytes).await.is_ok() {
                    ws_loop(session, msg_stream, app_data).await
                } else {
                    Ok(())
                }
            }
            Message::Text(s) => {
                tracing::debug!("Got text, {}", s);
                if session.text(s.to_string()).await.is_ok() {
                    let _ = app_data
                        .credentials_repo
                        .load(&UserId::from_str("1").unwrap())
                        .await;
                    ws_loop(session, msg_stream, app_data).await
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        },
        Some(Err(e)) => Err(e.to_string()),
        None => Ok(()),
    }
}

pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    app_data: web::Data<AppData>,
    // auth: BearerAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;
    // req.headers().
    // let credentials_repo = app_data.credentials_repo.clone();

    let _ = actix_rt::spawn(async move {
        let res = ws_loop(session.clone(), msg_stream, &app_data).await;
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
