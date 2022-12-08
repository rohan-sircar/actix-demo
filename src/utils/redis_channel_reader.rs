use std::sync::Arc;

use redis::{
    aio::ConnectionManager,
    streams::{StreamId, StreamReadOptions, StreamReadReply},
    AsyncCommands,
};
use tokio::sync::RwLock;

use crate::errors::DomainError;

#[derive(Clone, new)]
pub struct RedisChannelReader {
    channel_name: String,
    pub conn: ConnectionManager,
    pub last_msg_id: Arc<RwLock<Option<String>>>,
}

impl RedisChannelReader {
    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }

    pub async fn get_items(&self) -> Result<Vec<StreamId>, DomainError> {
        let mut conn = self.conn.clone();
        let opts = StreamReadOptions::default().block(0).count(5);
        let id = {
            let mb_id = self.last_msg_id.read().await;
            mb_id.clone().unwrap_or_else(|| "0".to_string())
        };
        let _ = tracing::debug!("Id = {id}");
        let rep: StreamReadReply = conn
            .xread_options(&[&self.channel_name], &[&id], &opts)
            .await?;
        let _ = tracing::debug!("Received keys {:?}", &rep.keys);
        let items = rep
            .keys
            .into_iter()
            .flat_map(|x| x.ids.into_iter())
            .collect::<Vec<StreamId>>();
        let _ = if let Some(x) = items.last() {
            let mut lmi = self.last_msg_id.write().await;
            *lmi = Some(x.id.clone());
        };
        Ok(items)
    }
}
