use redis::{
    aio::ConnectionManager,
    streams::{StreamId, StreamReadOptions, StreamReadReply},
    AsyncCommands,
};

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

#[derive(Serialize, Debug, Clone)]
pub enum RedisReply<T> {
    Success { id: String, data: T },
    Error { id: String, cause: String },
}

pub type RedisStreamResult<T> = Result<RedisReply<T>, DomainError>;

#[derive(Clone, new)]
pub struct RedisChannelReader<T> {
    channel_name: String,
    pub conn: ConnectionManager,
    last_msg_id: Option<String>,
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

    pub async fn get_items2(
        &mut self,
    ) -> Result<Vec<RedisReply<T>>, DomainError> {
        let mut conn = self.conn.clone();
        let opts = StreamReadOptions::default().block(0).count(5);
        let id = self.last_msg_id.clone().unwrap_or_else(|| "0".to_string());
        let _ = tracing::debug!("Id = {id}");
        let rep: StreamReadReply = conn
            .xread_options(&[&self.channel_name], &[&id], &opts)
            .await?;
        let _ = tracing::debug!("Received keys {:?}", &rep.keys);
        let items = rep
            .keys
            .into_iter()
            .flat_map(|x| x.ids.into_iter())
            .map(|m| {
                let msg = m.get::<String>("message").unwrap();
                match serde_json::from_str::<T>(&msg) {
                    Ok(msg) => RedisReply::Success {
                        id: m.id,
                        data: msg,
                    },
                    Err(err) => RedisReply::Error {
                        id: m.id,
                        cause: format!("Error parsing json - {err}"),
                    },
                }
                // .map_err(|err| {
                //     DomainError::new_internal_error(format!(
                //         "Error parsing json - {err}"
                //     ))
                // })
                // .map(|m2| )
            })
            .collect::<Vec<_>>();
        let _ = if let Some(x) = items.last() {
            match x {
                RedisReply::Success { id, data: _ } => {
                    self.last_msg_id = Some(id.clone());
                }
                RedisReply::Error { id, cause: _ } => {
                    self.last_msg_id = Some(id.clone());
                }
            }
        };
        Ok(items)
    }

    pub async fn get_items(&mut self) -> Result<Vec<StreamId>, DomainError> {
        let mut conn = self.conn.clone();
        let opts = StreamReadOptions::default().block(0).count(5);
        let id = self.last_msg_id.clone().unwrap_or_else(|| "0".to_string());
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
            self.last_msg_id = Some(x.id.clone());
        };
        Ok(items)
    }
}
