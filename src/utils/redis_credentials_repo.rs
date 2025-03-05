use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::DomainError;
use crate::models::users::UserId;

#[derive(new, Clone)]
pub struct RedisCredentialsRepo {
    base_key: String,
    redis: ConnectionManager,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionInfo {
    pub device_id: String,
    pub device_name: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub last_used_at: chrono::NaiveDateTime,
}

impl RedisCredentialsRepo {
    pub fn get_key(&self, user_id: &UserId) -> String {
        format!("{}.{user_id}", self.base_key)
    }

    // We'll use a separate key for tracking token expiration
    pub fn get_expiry_key(&self, user_id: &UserId, token: &str) -> String {
        format!("{}.{user_id}.expiry.{token}", self.base_key)
    }

    // For backward compatibility
    // pub async fn load(
    //     &self,
    //     user_id: &UserId,
    // ) -> Result<Option<String>, DomainError> {
    //     let sessions = self.load_all_sessions(user_id).await?;
    //     // Return the first token if any exists
    //     Ok(sessions.keys().next().map(|s| s.to_string()))
    // }

    // Method to check if a token is expired
    pub async fn is_token_expired(
        &self,
        user_id: &UserId,
        token: &str,
    ) -> Result<bool, DomainError> {
        let expiry_key = self.get_expiry_key(user_id, token);
        let exists: bool =
            self.redis.clone().exists(expiry_key).await.map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "Failed to check if expiry key exists: {err}"
                ))
            })?;
        Ok(exists)
    }

    // Load a specific session by token
    pub async fn load_session(
        &self,
        user_id: &UserId,
        token: &str,
    ) -> Result<Option<SessionInfo>, DomainError> {
        let key = self.get_key(user_id);
        let session_info_str: Option<String> =
            self.redis.clone().hget(key, token).await.map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to get session from Redis: {err}"
                ))
            })?;

        match session_info_str {
            Some(info_str) => {
                let session_info: SessionInfo = serde_json::from_str(&info_str)
                    .map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to deserialize session info: {err}"
                        ))
                    })?;
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

    // For backward compatibility
    // pub async fn save(
    //     &self,
    //     user_id: &UserId,
    //     jwt: &str,
    //     ttl_seconds: u64,
    // ) -> Result<(), DomainError> {
    //     let session_info = SessionInfo {
    //         device_id: Uuid::new_v4().to_string(),
    //         device_name: None,
    //         created_at: chrono::Utc::now().timestamp(),
    //         last_used_at: chrono::Utc::now().timestamp(),
    //     };

    //     self.save_session(user_id, jwt, &session_info, ttl_seconds)
    //         .await
    // }

    // Modified save_session method
    pub async fn save_session(
        &self,
        user_id: &UserId,
        token: &str,
        session_info: &SessionInfo,
        ttl_seconds: u64,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);

        // Serialize session info
        let session_info_str =
            serde_json::to_string(session_info).map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to serialize session info: {err}"
                ))
            })?;

        // Add to hash (without expiry)
        self.redis
            .clone()
            .hset::<String, &str, String, ()>(key, token, session_info_str)
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to save session to Redis: {err}"
                ))
            })?;

        // Set expiry for this specific token using a separate key
        let expiry_key = self.get_expiry_key(user_id, token);
        self.redis
            .clone()
            .set_ex::<String, &str, ()>(expiry_key, "1", ttl_seconds)
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to set expiry on Redis key: {err}"
                ))
            })?;

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
    pub async fn update_session_last_used(
        &self,
        user_id: &UserId,
        token: &str,
    ) -> Result<(), DomainError> {
        let session = self.load_session(user_id, token).await?;

        if let Some(mut session_info) = session {
            session_info.last_used_at = chrono::Utc::now().naive_utc();

            // Always extend by 30 minutes (1800 seconds)
            let ttl_seconds: u64 = 1800;

            // Update the session info and refresh the expiry
            self.save_session(user_id, token, &session_info, ttl_seconds)
                .await?;
        }

        Ok(())
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
