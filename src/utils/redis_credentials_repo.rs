use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::users::UserId;

#[derive(new, Clone)]
pub struct RedisCredentialsRepo {
    base_key: String,
    redis: ConnectionManager,
    max_sessions: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub device_id: String,
    pub device_name: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub last_used_at: chrono::NaiveDateTime,
    #[serde(skip)] // Skip serialization/deserialization
    pub ttl_remaining: Option<i64>,
}

#[derive(PartialEq)]
pub enum SessionStatus {
    Expired,
    Alive,
}
impl SessionStatus {
    pub fn from_exists(exists: bool) -> SessionStatus {
        if exists {
            SessionStatus::Alive
        } else {
            SessionStatus::Expired
        }
    }
}

impl RedisCredentialsRepo {
    pub fn get_key(&self, user_id: &UserId) -> String {
        format!("{}.{user_id}", self.base_key)
    }

    // We'll use a separate key for tracking token expiration
    pub fn get_expiry_key(&self, user_id: &UserId, token: &str) -> String {
        format!("{}.{user_id}.expiry.{token}", self.base_key)
    }

    // Method to check if a token is expired
    pub async fn is_token_expired(
        &self,
        user_id: &UserId,
        token: &str,
    ) -> Result<SessionStatus, DomainError> {
        let expiry_key = self.get_expiry_key(user_id, token);
        let exists: bool =
            self.redis.clone().exists(expiry_key).await.map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "Failed to check if expiry key exists: {err}"
                ))
            })?;
        // let sta
        Ok(SessionStatus::from_exists(exists))
    }

    // Load a specific session by token
    pub async fn load_session(
        &self,
        user_id: &UserId,
        token: &str,
    ) -> Result<Option<SessionInfo>, DomainError> {
        let session_key = self.get_key(user_id);
        let expiry_key = self.get_expiry_key(user_id, token);

        let mut pipe = redis::pipe();
        pipe.hget(&session_key, token).ttl(&expiry_key);
        let (mb_session_info_str, ttl): (Option<String>, i64) = pipe
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to get session info and TTL: {err}"
                ))
            })?;

        match mb_session_info_str {
            Some(info_str) => {
                let mut session_info: SessionInfo =
                    serde_json::from_str(&info_str).map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to deserialize session info: {err}"
                        ))
                    })?;

                // add time left till expiry
                session_info.ttl_remaining = Some(ttl);
                Ok(Some(session_info))
            }
            None => Ok(None),
        }
    }

    // Load all sessions for a user
    pub async fn load_all_sessions(
        &self,
        user_id: &UserId,
    ) -> Result<HashMap<String, SessionInfo>, DomainError> {
        let key = self.get_key(user_id);
        let sessions: HashMap<String, String> =
            self.redis.clone().hgetall(key).await.map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to get sessions from Redis: {err}"
                ))
            })?;

        let mut result = HashMap::new();
        for (token, session_info_str) in sessions {
            let session_info: SessionInfo =
                serde_json::from_str(&session_info_str).map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to deserialize session info: {err}"
                    ))
                })?;
            result.insert(token, session_info);
        }

        Ok(result)
    }

    // Modified save_session method
    pub async fn save_session(
        &self,
        user_id: &UserId,
        token: &str,
        session_info: &SessionInfo,
        ttl_seconds: u64,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);
        // Get expiry key
        let expiry_key = self.get_expiry_key(user_id, token);

        // Get current session count and attempt to add new session
        let current_count: i64 =
            self.redis.clone().hlen(&key).await.map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Redis pipeline failed: {err}"
                ))
            })?;

        let _ = tracing::info!("User has {current_count} sessions currently");

        // Check if limit exceeded
        if current_count >= self.max_sessions as i64 {
            return Err(DomainError::new_rate_limit_error(format!(
                "Maximum concurrent sessions ({}) exceeded",
                self.max_sessions
            )));
        }

        // Serialize session info
        let session_info_str =
            serde_json::to_string(session_info).map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to serialize session info: {err}"
                ))
            })?;

        // Create a pipeline
        // First check the TTL of the expiry key
        let mut pipe = redis::pipe();

        pipe.atomic()
            .hset(key, token, session_info_str)
            .ttl(&expiry_key);

        let (_, ttl): ((), i64) = pipe
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Error while trying to save the session: {err}"
                ))
            })?;

        // TTL returns -2 if key doesn't exist, -1 if no expiry, or remaining TTL in seconds
        if ttl > 0 {
            // Calculate new TTL
            let existing_ttl = ttl;
            tracing::debug!(
                "Existing TTL for key {expiry_key}: {existing_ttl} seconds",
            );
            let new_ttl: i64 = existing_ttl + ttl_seconds as i64;

            tracing::debug!(
                "Setting new TTL for key {expiry_key}: {new_ttl} seconds",
            );

            // Update expiry
            let () = self
                .redis
                .clone()
                .expire(expiry_key, new_ttl)
                .await
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to update expiry on Redis key: {err}"
                    ))
                })?;
        } else {
            // Set expiry for the first time
            let () = self
                .redis
                .clone()
                // exact value of the key is not important, we just need to set the expiry
                .set_ex(expiry_key, "1", ttl_seconds)
                .await
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to set expiry on Redis key: {err}"
                    ))
                })?;
        };

        Ok(())
    }

    // Delete a specific session
    pub async fn delete_session(
        &self,
        user_id: &UserId,
        token: &str,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);
        self.redis
            .clone()
            .hdel::<String, &str, ()>(key, token)
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to delete session from Redis: {err}"
                ))
            })?;

        Ok(())
    }

    // Delete all sessions for a user
    pub async fn delete_all_sessions(
        &self,
        user_id: &UserId,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);
        self.redis
            .clone()
            .del::<String, ()>(key)
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to delete sessions from Redis: {err}"
                ))
            })?;

        Ok(())
    }

    // Update last used time for a session
    pub async fn update_session_last_used_ws(
        &self,
        user_id: &UserId,
        token: &str,
        refresh_ttl_seconds: u64,
    ) -> Result<(), DomainError> {
        let mb_session_info = self.load_session(user_id, token).await?;

        if let Some(session_info) = mb_session_info {
            self.update_session_last_used(
                session_info,
                user_id,
                token,
                refresh_ttl_seconds,
            )
            .await?;
        }

        Ok(())
    }

    // Update last used time for a session
    pub async fn update_session_last_used(
        &self,
        mut session_info: SessionInfo,
        user_id: &UserId,
        token: &str,
        refresh_ttl_seconds: u64,
    ) -> Result<SessionInfo, DomainError> {
        session_info.last_used_at = chrono::Utc::now().naive_utc();

        // Update the session info and refresh the expiry
        self.save_session(user_id, token, &session_info, refresh_ttl_seconds)
            .await?;

        Ok(session_info)
    }

    // Add a cleanup method to be called periodically or during token validation
    pub async fn cleanup_expired_tokens(
        &self,
        user_id: &UserId,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);

        // Get all tokens for this user
        let tokens: Vec<String> =
            self.redis.clone().hkeys(key.clone()).await.map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to get tokens from Redis: {err}"
                ))
            })?;

        // Check each token's expiry key
        for token in tokens {
            let expiry_key = self.get_expiry_key(user_id, &token);
            let exists: bool =
                self.redis.clone().exists(expiry_key).await.map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to check if expiry key exists: {err}"
                    ))
                })?;

            // If expiry key doesn't exist, token has expired
            if !exists {
                // Remove the token from the hash
                self.redis
                    .clone()
                    .hdel::<String, &str, ()>(key.clone(), &token)
                    .await
                    .map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to delete expired token: {err}"
                        ))
                    })?;
            }
        }

        Ok(())
    }
}
