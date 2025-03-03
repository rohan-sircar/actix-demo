use crate::{errors::DomainError, models::ws::WsServerEvent, utils};
use actix_ws::Session;
use futures::future::BoxFuture;

pub trait SessionExt {
    fn send_server_event(
        &mut self,
        event: WsServerEvent,
    ) -> BoxFuture<'_, Result<(), DomainError>>;
}

impl SessionExt for Session {
    fn send_server_event(
        &mut self,
        event: WsServerEvent,
    ) -> BoxFuture<'_, Result<(), DomainError>> {
        Box::pin(async move {
            let msg = utils::jstr(&event);
            self.text(msg).await.map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to send text message: {err}"
                ))
            })
        })
    }
}
