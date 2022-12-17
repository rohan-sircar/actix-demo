pub mod broadcast_demo;
pub mod credentials_repo;
pub mod in_memory_credentials_repo;
pub mod redis_channel_reader;
pub mod redis_credentials_repo;
pub mod regex;
use std::sync::Arc;

use redis::aio::ConnectionManager;
use redis::aio::PubSub;

use crate::errors::DomainError;
use crate::AppData;

pub use self::credentials_repo::*;
pub use self::in_memory_credentials_repo::*;
pub use self::redis_channel_reader::*;
pub use self::regex::*;
pub mod ws;

pub async fn get_pubsub(app_data: Arc<AppData>) -> Result<PubSub, DomainError> {
    let client = app_data.redis_conn_factory.clone().ok_or_else(|| {
        DomainError::new_uninitialized_error("redis not initialized".to_owned())
    })?;
    Ok(client.get_async_connection().await?.into_pubsub())
}

pub async fn get_redis_conn(
    app_data: Arc<AppData>,
) -> Result<ConnectionManager, DomainError> {
    let client = app_data.redis_conn_factory.clone().ok_or_else(|| {
        DomainError::new_uninitialized_error("redis not initialized".to_owned())
    })?;
    Ok(ConnectionManager::new(client).await?)
}

// pub fn from_str<'a, T, F>(value: &'a str, mk_default: F) -> T
// where
//     T: serde::Deserialize<'a>,
//     F: Fn(()) -> T,
// {
//     let res = serde_json::from_str(value);

//     res.unwrap_or_else(|err| {
//         tracing::error!("Error deserializing: {:?}", err);
//         mk_default(())
//     })
// }
