use actix_ws::Session;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tracing_futures::Instrument;

use crate::{
    errors::DomainError,
    models::{users::UserId, ws::WsClientEvent},
    AppData,
};

use crate::utils::ws;

/// Processes incoming WebSocket client messages and dispatches them to appropriate handlers.
///
/// # Arguments
/// * `ws_msg` - The WebSocket message received from client
/// * `session` - Active WebSocket session
/// * `conn` - Redis connection manager for pub/sub operations
/// * `user_id` - ID of the authenticated user
/// * `app_data` - Shared application data
///
/// # Returns
/// Returns `Ok(())` on successful processing or `DomainError` if any error occurs
///
/// # Errors
/// Returns `DomainError` if:
/// - Message handling fails
/// - Redis operations fail
/// - WebSocket operations fail
pub async fn process_client_msg(
    ws_msg: WsClientEvent,
    session: Session,
    conn: &mut ConnectionManager,
    user_id: UserId,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    let redis_prefix = &app_data.redis_prefix;
    tracing::debug!("Processing WebSocket message: {:?}", ws_msg);
    match ws_msg {
        WsClientEvent::SendMessage { receiver, message } => {
            tracing::info!(
                "Handling message from user {} to {}",
                user_id,
                receiver
            );
            ws::handle_send_message(
                session,
                conn,
                user_id,
                receiver,
                message,
                redis_prefix,
            )
            .await
        }
        WsClientEvent::Error { cause } => {
            let _ = tracing::error!("client indicated error {}", cause);
            Ok(())
        }
        WsClientEvent::SubscribeJob { job_id } => {
            tracing::info!("User {} subscribing to job {}", user_id, job_id);
            actix_rt::spawn(
                async move {
                    let res =
                        ws::handle_subscribe_job(session, job_id, app_data)
                            .await;
                    tracing::info!("Job subscription ended: {res:?}");
                }
                .instrument(tracing::info_span!("job_subscribe_loop")),
            );
            Ok(())
        }
    }
}
