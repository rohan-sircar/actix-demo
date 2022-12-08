use std::sync::Arc;

use futures::prelude::*;
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

    // #[async_recursion]
    // async fn subscribe_loop(
    //     mut conn: MultiplexedConnection,
    //     channel_name: &str,
    //     opts: &StreamReadOptions,
    // ) -> Result<Vec<StreamId>, DomainError> {
    //     let rep: StreamReadReply =
    //         conn.xread_options(&[channel_name], &["$"], opts).await?;
    //     let items = rep.keys.iter().flat_map(|x| x.ids.clone()).collect();
    //     Ok(items)
    // }

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
        let items = stream::iter(rep.keys.into_iter())
            .flat_map(|x| stream::iter(x.ids.into_iter()))
            .then(|x| async move {
                {
                    let mut lmi = self.last_msg_id.write().await;
                    *lmi = Some(x.id.clone());
                }
                x
            })
            .collect()
            .await;
        Ok(items)
    }
}

// #[async_recursion]
// async fn subscribe_loop2(
//     mut conn: MultiplexedConnection,
//     channel_name: &str,
//     opts: &StreamReadOptions,
// ) -> impl Stream<Item = StreamId> {
//     let rep: StreamReadReply =
//         conn.xread_options(&[channel_name], &["$"], opts).await?;
//     let items: Vec<StreamId> = rep
//         .keys
//         .into_iter()
//         .flat_map(|x| x.ids.into_iter())
//         .collect();
//     if !items.is_empty() {
//         let keystream = stream::select(
//             stream::iter(items),
//             subscribe_loop2(conn, channel_name, opts),
//         );
//         keystream
//     } else {
//         stream::empty()
//     }
//     // Ok(items)
// }

// #[async_recursion]
// pub async fn get_items_loop(
//     mut conn: ConnectionManager,
//     channel_name: &str,
//     mut last_msg_id: String,
// ) -> Result<Vec<StreamId>, DomainError> {
//     let opts = StreamReadOptions::default().block(0);
//     let rep: StreamReadReply = conn
//         .xread_options(&[channel_name], &[&last_msg_id], &opts)
//         .await?;
//     let items: Vec<StreamId> = rep
//         .keys
//         .into_iter()
//         .inspect(|x| {
//             last_msg_id = x.key.clone();
//         })
//         .flat_map(|x| x.ids)
//         .collect();
//     if items.is_empty() {
//         Ok(items)
//     } else {
//         Ok(get_items_loop(conn, channel_name, last_msg_id).await?)
//     }
//     // Ok(items)
// }
