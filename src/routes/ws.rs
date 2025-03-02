use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{errors::DomainError, utils, AppData};
use actix_http::header::HeaderMap;
use actix_rt::time::sleep;
use actix_web::{web, HttpRequest, HttpResponse};

use tracing_futures::Instrument;

use super::auth::get_claims;

#[tracing::instrument(level = "info", skip_all, fields(auth_user_id))]
pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, actix_web::Error> {
    let headers = req.headers();

    let token = extract_auth_token(headers)?;

    let claims = get_claims(&app_data.jwt_key, &token)?;

    let user_id = claims.custom.user_id;

    let _ = tracing::Span::current().record("auth_user_id", user_id.as_uint());

    let _ = tracing::info!("Initiating websocket connection");

    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    let _ = tracing::info!("Websocket connection initiated");

    let session2 = session.clone();
    let cm = utils::get_new_redis_conn(app_data.clone().into_inner()).await?;

    let _ = tracing::info!("Connected to Redis");

    let app_data2 = app_data.clone().into_inner();
    let handles = Rc::new(RefCell::new(Vec::new()));

    let _ = tracing::info!("Starting message receiver");
    let msg_receiver = Rc::new(actix_rt::spawn(
        async move {
            let res =
                utils::ws::msg_receive_loop(user_id, cm, session2, app_data2)
                    .await;
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
    ));
    let _ = {
        handles.borrow_mut().push(msg_receiver.clone());
    };

    let session2 = session.clone();
    let mut pub_cm =
        utils::get_new_redis_conn(app_data.clone().into_inner()).await?;
    let _ = tracing::info!("Connected to Redis PubSub");
    let ws_loop = Rc::new(actix_rt::spawn(
        async move {
            tracing::info!("Starting websocket loop");
            let res = utils::ws::ws_loop(
                session2.clone(),
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
                        "Websocket connection ended with error {err:?}"
                    );
                }
            };

            let _ = session2.close(None).await;
            let _ = msg_receiver.abort();
        }
        .instrument(tracing::info_span!("ws_loop")),
    ));
    let _ = {
        handles.borrow_mut().push(ws_loop);
    };
    let mut session2 = session.clone();
    let _hb = actix_rt::spawn(
        async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                if session2.ping(b"").await.is_err() {
                    for h in handles.borrow().iter() {
                        h.abort();
                    }
                    break;
                }
            }
        }
        .instrument(tracing::info_span!("ws_hb")),
    );

    Ok(response)
}

/// Extract the X-AUTH-TOKEN from the "cookie" header in the given HttpRequest.
pub fn extract_auth_token(headers: &HeaderMap) -> Result<String, DomainError> {
    // Get the raw cookie header string
    let header = headers
        .get("cookie")
        .and_then(|hv| hv.to_str().ok())
        .ok_or_else(|| {
            DomainError::new_bad_input_error(format!("Cookie header not set"))
        })?;

    let token = header
        .split(';')
        .map(|s| s.trim()) // Trim whitespace around each cookie token
        .filter_map(|cookie_str| {
            // Try to parse each cookie fragment
            awc::cookie::Cookie::parse_encoded(cookie_str.to_owned()).ok()
        })
        .find_map(|cookie| {
            if cookie.name() == "X-AUTH-TOKEN" {
                Some(cookie.value().to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            DomainError::new_auth_error("X-AUTH-TOKEN not found".to_owned())
        })?;

    Ok(token)
}
