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

pub async fn process_client_msg(
    ws_msg: WsClientEvent,
    session: Session,
    conn: &mut ConnectionManager,
    user_id: UserId,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    let redis_prefix = &app_data.redis_prefix;
    match ws_msg {
        WsClientEvent::SendMessage { receiver, message } => {
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
