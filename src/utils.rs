// pub mod broadcast_demo;
pub mod instrumented_redis_cache;
pub mod redis_channel_reader;
pub mod redis_credentials_repo;
pub mod regex;
pub mod ws;
pub use self::instrumented_redis_cache::InstrumentedRedisCache;

use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use actix_web::web;
use mime::Mime;

use actix_http::header::HeaderMap;
use futures::StreamExt;
use jwt_simple::claims::JWTClaims;
use jwt_simple::prelude::*;
use redis::aio::ConnectionManager;
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

pub async fn get_new_redis_conn(
    app_data: Arc<AppData>,
) -> Result<ConnectionManager, DomainError> {
    let client = app_data.redis_conn_factory.clone();
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

/// Validates an image stream by checking its MIME type and size
pub async fn validate_image_stream(
    mut payload: actix_web::web::Payload,
    content_type: &str,
    max_size_bytes: usize,
) -> Result<web::BytesMut, DomainError> {
    const HEAD_CHUNK_SIZE: usize = 512;

    // Parse and validate content type
    let mime_type: Mime = content_type.parse().map_err(|err| {
        DomainError::new_bad_input_error(format!("Invalid mime type: {}", err))
    })?;

    if !matches!(mime_type.type_(), mime::IMAGE) {
        return Err(DomainError::new_bad_input_error(
            "Only image files are allowed".to_string(),
        ));
    }

    // Read initial chunk for type detection
    let mut head_buffer = web::BytesMut::with_capacity(HEAD_CHUNK_SIZE);
    let mut full_file = web::BytesMut::new();

    while head_buffer.len() < HEAD_CHUNK_SIZE {
        match payload.next().await {
            Some(chunk_result) => {
                let chunk = chunk_result?;
                let needed = HEAD_CHUNK_SIZE - head_buffer.len();
                if chunk.len() > needed {
                    head_buffer.extend_from_slice(&chunk[..needed]);
                    full_file.extend_from_slice(&chunk);
                } else {
                    head_buffer.extend_from_slice(&chunk);
                    full_file.extend_from_slice(&chunk);
                }
            }
            None => break,
        }
    }

    // Validate file content using infer
    let mime_type_from_content = infer::get(&head_buffer).ok_or_else(|| {
        DomainError::new_bad_input_error(
            "Could not determine file type from content".to_string(),
        )
    })?;

    // Verify allowed image types
    if !matches!(
        mime_type_from_content.mime_type(),
        "image/jpeg" | "image/png" | "image/webp"
    ) {
        return Err(DomainError::InvalidMimeType {
            detected: mime_type_from_content.mime_type().to_string(),
        });
    }

    // Verify Content-Type matches actual file type
    if mime_type_from_content.mime_type() != mime_type.essence_str() {
        return Err(DomainError::new_bad_input_error(format!(
            "Content-Type {} does not match actual file type {}",
            mime_type.essence_str(),
            mime_type_from_content.mime_type()
        )));
    }

    // Continue reading the rest of the payload
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // Check size limit
        if (full_file.len() + chunk.len()) > max_size_bytes {
            return Err(DomainError::FileSizeExceeded {
                max_bytes: max_size_bytes as u64,
            });
        }
        full_file.extend_from_slice(&chunk);
    }

    Ok(full_file)
}

/// Extracts a header value as a String from the headers map
pub fn extract_header_value(
    headers: &HeaderMap,
    header_name: &str,
) -> Result<String, DomainError> {
    headers
        .get(header_name)
        .ok_or_else(|| {
            DomainError::new_bad_input_error(format!(
                "Missing {} header",
                header_name
            ))
        })
        .and_then(|hv| {
            hv.to_str().map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "{} header is not a valid UTF-8 string: {}",
                    header_name, err
                ))
            })
        })
        .map(|s| s.to_string())
}

pub fn extract_user_id_from_header(
    headers: &HeaderMap,
) -> Result<UserId, DomainError> {
    extract_header_value(headers, "x-auth-user").and_then(|str| {
        UserId::from_str(&str).map_err(|err| {
            DomainError::new_bad_input_error(format!(
                "Invalid UserId format in x-auth-user header: {err}"
            ))
        })
    })
}
