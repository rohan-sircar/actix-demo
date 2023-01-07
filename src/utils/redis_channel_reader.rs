use redis::{
    aio::ConnectionManager,
    streams::{StreamId, StreamReadOptions, StreamReadReply},
    AsyncCommands,
};

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

#[derive(Serialize, Debug, Clone)]
pub enum RedisReplyKind<T> {
    Ok { data: T },
    Error { cause: String },
}

#[derive(Serialize, Debug, Clone)]
pub struct RedisReply<T> {
    pub id: String,
    pub kind: RedisReplyKind<T>,
}

pub type RedisStreamResult<T> = Result<RedisReply<T>, DomainError>;

#[derive(new)]
pub struct RedisChannelReader<T> {
    channel_name: String,
    pub conn: ConnectionManager,
    last_msg_id: Option<String>,
    opts: StreamReadOptions,
    pd: std::marker::PhantomData<T>,
}

impl<T> RedisChannelReader<T>
where
    T: for<'a> Deserialize<'a>,
{
    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }

    pub fn last_msg_id(&self) -> Option<&str> {
        self.last_msg_id.as_deref()
    }

    pub async fn get_items(
        &mut self,
    ) -> Result<Vec<RedisReply<T>>, DomainError> {
        let mut conn = self.conn.clone();
        let id = self.last_msg_id.clone().unwrap_or_else(|| "0".to_string());
        let _ = tracing::trace!("Id = {id}");
        let rep: StreamReadReply = conn
            .xread_options(&[&self.channel_name], &[&id], &self.opts)
            .await?;
        let _ = tracing::trace!("Received keys {:?}", &rep.keys);
        let items = rep
            .keys
            .into_iter()
            .flat_map(|x| x.ids.into_iter())
            .map(|m| {
                let msg = m.get::<String>("message").unwrap();
                match serde_json::from_str::<T>(&msg) {
                    Ok(msg) => RedisReply {
                        id: m.id,
                        kind: RedisReplyKind::Ok { data: msg },
                    },
                    Err(err) => RedisReply {
                        id: m.id,
                        kind: RedisReplyKind::Error {
                            cause: format!("Error parsing json - {err}"),
                        },
                    },
                }
            })
            .collect::<Vec<_>>();
        let _ = if items.last().is_some() {
            self.last_msg_id = Some(id.clone());
        };
        Ok(items)
    }

    pub async fn get_items_unparsed(
        &mut self,
    ) -> Result<Vec<StreamId>, DomainError> {
        let mut conn = self.conn.clone();
        let id = self.last_msg_id.clone().unwrap_or_else(|| "0".to_string());
        let _ = tracing::debug!("Id = {id}");
        let rep: StreamReadReply = conn
            .xread_options(&[&self.channel_name], &[&id], &self.opts)
            .await?;
        let _ = tracing::debug!("Received keys {:?}", &rep.keys);
        let items = rep
            .keys
            .into_iter()
            .flat_map(|x| x.ids.into_iter())
            .collect::<Vec<_>>();
        let _ = if let Some(x) = items.last() {
            self.last_msg_id = Some(x.id.clone());
        };
        Ok(items)
    }
}
