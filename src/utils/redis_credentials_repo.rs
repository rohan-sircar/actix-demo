use prometheus::GaugeVec;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::session::{SessionInfo, SessionStatus};
use crate::models::users::UserId;

#[derive(new, Clone)]
pub struct RedisCredentialsRepo {
    base_key: String,
    redis: ConnectionManager,
    max_sessions: u8,
    refresh_ttl_seconds: u64,
    active_sessions: GaugeVec,
}

impl RedisCredentialsRepo {
    pub fn get_key(&self, user_id: &UserId) -> String {
        format!("{}.{user_id}", self.base_key)
    }

    // We'll use a separate key for tracking token expiration
    pub fn get_expiry_key(
        &self,
        user_id: &UserId,
        session_id: &Uuid,
    ) -> String {
        format!("{}.expiry.{user_id}.{session_id}", self.base_key)
    }

    // Method to check if a token is expired
    pub async fn is_token_expired(
        &self,
        user_id: &UserId,
        session_id: &Uuid,
    ) -> Result<SessionStatus, DomainError> {
        let expiry_key = self.get_expiry_key(user_id, session_id);
        let exists: bool =
            self.redis.clone().exists(expiry_key).await.map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to check if expiry key exists: {err}"
                ))
            })?;
        Ok(SessionStatus::from_exists(exists))
    }

    // Load a specific session by token
    pub async fn load_session(
        &self,
        user_id: &UserId,
        session_id: &Uuid,
    ) -> Result<Option<SessionInfo>, DomainError> {
        let session_key = self.get_key(user_id);
        let session_id_str = session_id.to_string();
        let expiry_key = self.get_expiry_key(user_id, session_id);

        let mut pipe = redis::pipe();
        pipe.hget(&session_key, &session_id_str).ttl(&expiry_key);
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
    ) -> Result<HashMap<Uuid, SessionInfo>, DomainError> {
        let key = self.get_key(user_id);
        let sessions: HashMap<String, String> =
            self.redis.clone().hgetall(key).await.map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to get sessions from Redis: {err}"
                ))
            })?;

        let mut result = HashMap::new();
        let mut pipe = redis::pipe();

        // First pass: deserialize session info and prepare TTL checks
        let mut expiry_keys = Vec::new();
        for (session_id, session_info_str) in sessions {
            let session_info: SessionInfo =
                serde_json::from_str(&session_info_str).map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to deserialize session info: {err}"
                    ))
                })?;
            let session_id = Uuid::parse_str(&session_id).unwrap();
            let expiry_key = self.get_expiry_key(user_id, &session_id);
            expiry_keys.push(expiry_key.clone());
            pipe.ttl(&expiry_key);
            result.insert(session_id, session_info);
        }

        // Execute batch TTL checks
        if !expiry_keys.is_empty() {
            let ttls: Vec<i64> = pipe
                .query_async(&mut self.redis.clone())
                .await
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to get batch TTLs: {err}"
                    ))
                })?;

            // Apply TTLs to corresponding sessions
            for (i, (_session_id, session_info)) in
                result.iter_mut().enumerate()
            {
                session_info.ttl_remaining = Some(ttls[i]);
            }
        }
        // Update active sessions metric for this user
        self.active_sessions
            .with_label_values(&[&user_id.to_string()])
            .set(result.len() as f64);

        Ok(result)
    }

    // Create a new session for a user. Will error if session already exists or max sessions exceeded.
    pub async fn create_session(
        &self,
        user_id: &UserId,
        session_id: &Uuid,
        session_info: &SessionInfo,
        ttl_seconds: u64,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);
        let session_id_str = session_id.to_string();
        let expiry_key = self.get_expiry_key(user_id, session_id);

        // Create pipeline to check session existence and get count
        let mut pipe = redis::pipe();
        pipe.atomic().hexists(&key, &session_id_str).hlen(&key);

        let (exists, current_count): (bool, i64) = pipe
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to check session existence and count: {err}"
                ))
            })?;

        if exists {
            return Err(DomainError::new_bad_input_error(
                "Session already exists".to_string(),
            ));
        }

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

        // Create a pipeline for atomic operations
        let mut pipe = redis::pipe();
        pipe.atomic()
            .hset(&key, &session_id_str, session_info_str)
            .set_ex(expiry_key, "1", ttl_seconds)
            .hlen(&key);

        let _ = tracing::info!("Creating user session");

        let ((), (), count): (_, _, i32) = pipe
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to create session: {err}"
                ))
            })?;

        // Update active sessions metric for this user
        self.active_sessions
            .with_label_values(&[&user_id.to_string()])
            .set(count as f64);

        Ok(())
    }

    // Update an existing session. Will error if session does not exist.
    pub async fn update_session(
        &self,
        user_id: &UserId,
        session_id: &Uuid,
        session_info: &SessionInfo,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);
        let session_id_str = session_id.to_string();
        let expiry_key = self.get_expiry_key(user_id, session_id);

        // Check if session exists
        let exists: bool = self
            .redis
            .clone()
            .hexists(&key, &session_id_str)
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to check if session exists: {err}"
                ))
            })?;

        if !exists {
            return Err(DomainError::new_bad_input_error(
                "Session does not exist".to_string(),
            ));
        }

        // Serialize session info
        let session_info_str =
            serde_json::to_string(session_info).map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to serialize session info: {err}"
                ))
            })?;

        // Create a pipeline for atomic operations
        let mut pipe = redis::pipe();

        pipe.atomic()
            .hset(key, &session_id_str, session_info_str)
            .ttl(&expiry_key);

        let _ = tracing::info!("Updating user session");

        let (_, ttl): ((), i64) = pipe
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to update session: {err}"
                ))
            })?;

        // TTL returns -2 if key doesn't exist, -1 if no expiry, or remaining TTL in seconds
        if ttl < 0 {
            return Err(DomainError::AuthError {
                message: format!("Session has expired - ttl: {ttl}"),
            });
        }

        // Calculate new TTL
        let existing_ttl = ttl;
        let _ = tracing::debug!(
            "Existing TTL for key {expiry_key}: {existing_ttl} seconds",
        );
        let new_ttl: i64 = existing_ttl + self.refresh_ttl_seconds as i64;

        let _ = tracing::debug!(
            "Setting new TTL for key {expiry_key}: {new_ttl} seconds",
        );

        let _ = tracing::info!("Extending user session");

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

        Ok(())
    }

    // Update last used time for a session
    pub async fn update_session_last_used_ws(
        &self,
        user_id: &UserId,
        session_id: &Uuid,
    ) -> Result<(), DomainError> {
        let mb_session_info = self.load_session(user_id, session_id).await?;

        if let Some(session_info) = mb_session_info {
            self.update_session_last_used(session_id, session_info, user_id)
                .await?;
        }

        Ok(())
    }

    // Update last used time for a session
    pub async fn update_session_last_used(
        &self,
        session_id: &Uuid,
        mut session_info: SessionInfo,
        user_id: &UserId,
    ) -> Result<SessionInfo, DomainError> {
        session_info.last_used_at = chrono::Utc::now().naive_utc();

        // Update the session info and refresh the expiry
        self.update_session(user_id, session_id, &session_info)
            .await?;

        Ok(session_info)
    }

    // Delete a specific session
    pub async fn delete_session(
        &self,
        user_id: &UserId,
        session_id: &Uuid,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);
        let session_id_str = session_id.to_string();

        let mut pipe = redis::pipe();
        pipe.atomic().hdel(&key, &session_id_str).hlen(&key);

        let (_, count): ((), i32) = pipe
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to delete session from Redis: {err}"
                ))
            })?;

        // Update active sessions metric for this user
        self.active_sessions
            .with_label_values(&[&user_id.to_string()])
            .set(count as f64);

        Ok(())
    }

    // Delete all sessions for a user
    pub async fn delete_all_sessions(
        &self,
        user_id: &UserId,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);

        let mut pipe = redis::pipe();
        pipe.atomic().del(&key).hlen(&key);

        let (_, count): ((), i32) = pipe
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to delete sessions from Redis: {err}"
                ))
            })?;

        // Update active sessions metric for this user
        self.active_sessions
            .with_label_values(&[&user_id.to_string()])
            .set(count as f64);

        Ok(())
    }

    // Add a cleanup method to be called periodically or during token validation
    pub async fn cleanup_expired_session_ids(
        &self,
        user_id: &UserId,
    ) -> Result<(), DomainError> {
        let key = self.get_key(user_id);

        // Get all session_ids for this user
        let session_ids: Vec<String> =
            self.redis.clone().hkeys(&key).await.map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to get session_ids from Redis: {err}"
                ))
            })?;

        let mut pipe = redis::pipe();
        let mut expired_count = 0;

        // Check each token's expiry key and queue deletions
        for session_id_str in session_ids {
            let session_id = Uuid::parse_str(&session_id_str)
                .expect("Expected valid session_id");
            let expiry_key = self.get_expiry_key(user_id, &session_id);
            let exists: bool =
                self.redis.clone().exists(expiry_key).await.map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to check if expiry key exists: {err}"
                    ))
                })?;

            // If expiry key doesn't exist, token has expired
            if !exists {
                pipe.hdel(&key, &session_id_str);
                expired_count += 1;
            }
        }

        // Execute batch deletions if any expired sessions found
        if expired_count > 0 {
            pipe.atomic().hlen(&key);
            let count: i32 = pipe
                .query_async(&mut self.redis.clone())
                .await
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to delete expired sessions: {err}"
                    ))
                })?;

            // Update active sessions metric for this user
            self.active_sessions
                .with_label_values(&[&user_id.to_string()])
                .set(count as f64);

            let _ = tracing::info!("Removed {expired_count} expired sessions, {count} active sessions remaining");
        }

        Ok(())
    }
}
