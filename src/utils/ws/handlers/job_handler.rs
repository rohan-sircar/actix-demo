use actix_ws::Session;
use futures::StreamExt;
use std::sync::Arc;

use crate::{
    errors::DomainError,
    models::ws::{MyProcessItem, WsServerEvent},
    utils, AppData,
};

pub async fn subscribe_job(
    mut session: Session,
    chan_name: String,
    app_data: Arc<AppData>,
) -> Result<(), DomainError> {
    let mut ps = utils::get_pubsub(app_data).await?;
    let _ = ps.subscribe(&chan_name).await?;
    {
        let mut msg_stream = ps.on_message();
        while let Some(msg) = msg_stream.next().await {
            let cmd = msg.get_payload::<String>().unwrap_or_default();
            let _ = tracing::debug!("Got cmd {cmd}");
            let rcm = match serde_json::from_str::<MyProcessItem>(&cmd) {
                Ok(rcm) => rcm,
                Err(_) => {
                    tracing::error!("Failed to parse command: {cmd}");
                    continue;
                }
            };
            let server_msg = WsServerEvent::CommandMessage {
                message: rcm.clone(),
            };

            let msg_str = utils::jstr(&server_msg);

            let should_break = match &rcm {
                MyProcessItem::Line { .. } | MyProcessItem::Error { .. } => {
                    session.text(msg_str).await.is_err()
                }
                MyProcessItem::Done { code } => {
                    let _ =
                        tracing::info!("Process completed with code={code}");
                    let _ = session.text(msg_str).await;
                    true
                }
            };

            if should_break {
                break;
            }
        }
    }
    ps.unsubscribe(&chan_name).await?;
    Ok(())
}
