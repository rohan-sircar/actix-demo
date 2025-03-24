// pub mod broadcast_demo;
pub mod redis_channel_reader;
pub mod redis_credentials_repo;
pub mod regex;
pub mod ws;

use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use actix_http::header::HeaderMap;
use jwt_simple::claims::JWTClaims;
use jwt_simple::prelude::*;
use redis::aio::ConnectionManager;
use redis::aio::PubSub;
use serde::Serialize;

use crate::errors::DomainError;
use crate::models::users::UserId;
use crate::routes::auth::VerifiedAuthDetails;
use crate::AppData;

mod rate_limit_backend;
pub use self::rate_limit_backend::RateLimitBackend;

pub use self::redis_channel_reader::*;
pub use self::regex::*;
pub use self::ws::{msg_receive_loop, ws_loop};

mod cookie_auth;
pub use cookie_auth::{cookie_auth, extract_auth_token, CookieAuth};

pub async fn get_pubsub(app_data: Arc<AppData>) -> Result<PubSub, DomainError> {
    let client = app_data.redis_conn_factory.clone().ok_or_else(|| {
        DomainError::new_uninitialized_error("redis not initialized".to_owned())
    })?;
    Ok(client.get_async_pubsub().await?)
}

pub async fn get_new_redis_conn(
    app_data: Arc<AppData>,
) -> Result<ConnectionManager, DomainError> {
    let client = app_data.redis_conn_factory.clone().ok_or_else(|| {
        DomainError::new_uninitialized_error("redis not initialized".to_owned())
    })?;
    Ok(ConnectionManager::new(client).await?)
}

pub fn get_redis_prefix<T: Display>(
    prefix: T,
) -> impl Fn(&dyn Display) -> String {
    move |st| format!("{prefix}.{st}")
}

pub fn jstr<T>(value: &T) -> String
where
    T: ?Sized + Serialize,
{
    serde_json::to_string(value).expect("failed to serialize {value}")
}

pub fn from_str<'a, T, F>(value: &'a str, mk_default: F) -> T
where
    T: serde::Deserialize<'a>,
    F: Fn(()) -> T,
{
    let res = serde_json::from_str(value);

    res.unwrap_or_else(|err| {
        tracing::error!("Error deserializing: {:?}", err);
        mk_default(())
    })
}

pub fn get_claims(
    jwt_key: &HS256Key,
    token: &str,
) -> Result<JWTClaims<VerifiedAuthDetails>, DomainError> {
    jwt_key
        .verify_token::<VerifiedAuthDetails>(token, None)
        .map_err(|err| DomainError::anyhow_auth("Failed to verify token", err))
}

pub fn extract_user_id_from_header(
    headers: &HeaderMap,
) -> Result<UserId, DomainError> {
    headers
        .get("x-auth-user")
        .ok_or_else(|| {
            DomainError::new_auth_error("Missing x-auth-user header".to_owned())
        })
        .and_then(|hv| {
            hv.to_str().map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "x-auth-user header is not a valid UTF-8 string: {err}"
                ))
            })
        })
        .and_then(|str| {
            UserId::from_str(str).map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "Invalid UserId format in x-auth-user header: {err}"
                ))
            })
        })
}
