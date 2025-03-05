use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{errors::DomainError, utils, AppData};
use actix_http::header::HeaderMap;
use actix_rt::time::sleep;
use actix_web::{web, HttpRequest, HttpResponse};

use tracing_futures::Instrument;

use super::auth::validate_token;

/// Handles incoming WebSocket connections
///
/// # Flow
/// 1. Extracts and validates authentication token from request headers
/// 2. Establishes WebSocket connection
/// 3. Spawns three concurrent workers:
///    - Message receiver: Handles incoming messages from Redis
///    - WebSocket loop: Manages WebSocket communication
///    - Heartbeat: Maintains connection health
/// 4. Returns HTTP response with established WebSocket connection
#[tracing::instrument(level = "info", skip_all, fields(auth_user_id))]
pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, actix_web::Error> {
    let headers = req.headers();

    let _ = tracing::debug!("Extracting auth token from headers");
    let token = extract_auth_token(headers)?;
    let _ = tracing::debug!("Successfully extracted auth token");

    // Validate token using existing logic
    let credentials_repo = &app_data.credentials_repo;
    let jwt_key = &app_data.jwt_key;

    let _ = tracing::debug!("Validating user session");

    let _ = validate_token(credentials_repo, jwt_key, token.clone()).await?;

    let _ = tracing::debug!("Validating JWT claims");
    let claims = utils::get_claims(&app_data.jwt_key, &token)?;
    let user_id = claims.custom.user_id;

    let _ =
        tracing::debug!("Successfully validated claims for user {}", user_id);

    let _ = tracing::Span::current().record("auth_user_id", user_id.as_uint());

    let _ =
        tracing::info!("Initiating websocket connection for user {}", user_id);

    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    let _ =
        tracing::info!("Websocket connection established for user {}", user_id);

    let session2 = session.clone();
    let cm = utils::get_new_redis_conn(app_data.clone().into_inner()).await?;

    let _ = tracing::info!("Connected to Redis");

    let app_data2 = app_data.clone().into_inner();
    let handles = Rc::new(RefCell::new(Vec::new()));

    let _ = tracing::info!("Starting message receiver for user {}", user_id);
    let msg_receiver = Rc::new(actix_rt::spawn(
        async move {
            let _ = tracing::debug!("Entering message receive loop");
            let res =
                utils::ws::msg_receive_loop(user_id, cm, session2, app_data2)
                    .await;
            let _ = match res {
                Ok(_) => {
                    let _ = tracing::info!("Message receive loop ended successfully for user {}", user_id);
                }
                Err(err) => {
                    let _ = tracing::error!(
                        "Message receive loop ended with error for user {}: {:?}",
                        user_id,
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
    let _ = tracing::info!("Connected to Redis PubSub for user {}", user_id);

    // Handles WebSocket communication with the client
    //
    // Responsibilities:
    // 1. Processes incoming WebSocket messages
    // 2. Handles graceful shutdown on errors
    let ws_loop = Rc::new(actix_rt::spawn(
        async move {
            tracing::info!("Starting websocket loop for user {}", user_id);
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
                        "Websocket connection ended successfully for user {}",
                        user_id
                    );
                }
                Err(err) => {
                    let _ = tracing::error!(
                        "Websocket connection ended with error for user {}: {err:?}",
                        user_id
                    );
                }
            };

            let _ = tracing::debug!("Closing WebSocket session for user {}", user_id);
            let _ = session2.close(None).await;
            let _ = tracing::debug!("Aborting message receiver for user {}", user_id);
            let _ = msg_receiver.abort();
        }
        .instrument(tracing::info_span!("ws_loop")),
    ));
    let _ = {
        handles.borrow_mut().push(ws_loop);
    };

    let mut session2 = session.clone();
    // Maintains WebSocket connection health
    //
    // Responsibilities:
    // 1. Sends periodic ping messages to client
    // 2. Detects connection failures
    // 3. Cleans up resources on connection loss
    let _hb = actix_rt::spawn(
        async move {
            let _ = tracing::debug!("Starting heartbeat for user {}", user_id);
            loop {
                sleep(Duration::from_secs(30)).await;
                if session2.ping(b"").await.is_err() {
                    let _ = tracing::warn!(
                        "Heartbeat failed for user {}, cleaning up resources",
                        user_id
                    );
                    for h in handles.borrow().iter() {
                        h.abort();
                    }
                    break;
                }
            }
            let _ = tracing::debug!("Heartbeat stopped for user {}", user_id);
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
            DomainError::new_bad_input_error("Cookie header not set".to_owned())
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
