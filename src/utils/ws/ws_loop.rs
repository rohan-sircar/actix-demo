use std::sync::Arc;

use crate::{
    errors::DomainError, models::users::UserId, models::ws::WsClientEvent,
    models::ws::WsServerEvent, utils, AppData,
};

use utils::ws;

use actix_ws::{Message, MessageStream, Session};
use futures::StreamExt;
use redis::aio::ConnectionManager;

/// Guard for tracking WebSocket connections
struct WsConnectionGuard<'a> {
    metrics: &'a prometheus::GaugeVec,
    user_id: String,
}

impl<'a> WsConnectionGuard<'a> {
    fn new(metrics: &'a prometheus::GaugeVec, user_id: UserId) -> Self {
        let _ = metrics.with_label_values(&[&user_id.to_string()]).inc();
        Self {
            metrics,
            user_id: user_id.to_string(),
        }
    }
}

impl<'a> Drop for WsConnectionGuard<'a> {
    fn drop(&mut self) {
        let _ = self.metrics.with_label_values(&[&self.user_id]).dec();
    }
}

/// Main WebSocket message processing loop
///
/// Handles incoming WebSocket messages and processes them accordingly.
/// Manages the WebSocket session and connection state.
///
/// # Arguments
/// * `session` - WebSocket session
/// * `msg_stream` - Stream of incoming WebSocket messages
/// * `conn` - Redis connection manager
/// * `user_id` - ID of the connected user
/// * `app_data` - Shared application data
///
/// # Returns
/// Result indicating success or failure of the WebSocket session
pub async fn ws_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
    conn: &mut ConnectionManager,
    user_id: UserId,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    let _guard = WsConnectionGuard::new(
        &app_data.metrics.active_ws_connections,
        user_id,
    );
    tracing::info!("Starting WebSocket loop for user {}", user_id);

    while let Some(item) = msg_stream.next().await {
        match item {
            Ok(Message::Ping(bytes)) => {
                tracing::debug!("Received ping message");
                if session.pong(&bytes).await.is_err() {
                    tracing::warn!("Failed to send pong, closing connection");
                    break;
                }
            }
            Ok(Message::Text(s)) => {
                tracing::info!("Received text message from user {}", user_id);
                tracing::debug!("Message content: {}", s);

                let res = match serde_json::from_str::<WsClientEvent>(&s) {
                    Ok(ws_msg) => {
                        tracing::debug!(
                            "Processing client message: {:?}",
                            ws_msg
                        );
                        Ok(ws::process_client_msg(
                            ws_msg,
                            session.clone(),
                            conn,
                            user_id,
                            app_data.clone(),
                        )
                        .await?)
                    }
                    Err(err) => {
                        tracing::warn!("Failed to parse message: {}", err);
                        let err = &WsServerEvent::Error {
                            id: None,
                            cause: err.to_string(),
                        };
                        let err = utils::jstr(err);
                        session.text(err).await
                    }
                };

                if res.is_err() {
                    tracing::warn!(
                        "Error processing message, closing connection"
                    );
                    break;
                }
            }
            Ok(Message::Close(reason)) => {
                tracing::info!("Received close message, reason={:?}", reason);
                break;
            }
            Ok(_) => {
                tracing::warn!(
                    "Received unexpected message type, closing connection"
                );
                break;
            }
            Err(_) => {
                tracing::warn!("Error receiving message, closing connection");
                break;
            }
        }
    }

    tracing::info!("WebSocket loop ended for user {}", user_id);
    Ok(())
}
