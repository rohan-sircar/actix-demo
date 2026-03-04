use prometheus::GaugeVec;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::session::{SessionInfo, SessionStatus};
use crate::models::users::UserId;

// Lua script for atomic session creation
const CREATE_SESSION_LUA: &str = r#"
local key = KEYS[1]
local session_id_str = ARGV[1]
local session_info_str = ARGV[2]
local expiry_key = ARGV[3]
local ttl_seconds = tonumber(ARGV[4])
local max_sessions = tonumber(ARGV[5])

-- Check if session already exists
if redis.call('HEXISTS', key, session_id_str) == 1 then
    return {0, 'Session already exists'}
end

-- Check session count
local current_count = redis.call('HLEN', key)
if current_count >= max_sessions then
    return {0, 'Maximum concurrent sessions exceeded'}
end

-- Create session
redis.call('HSET', key, session_id_str, session_info_str)
redis.call('SETEX', expiry_key, ttl_seconds, '1')

return {1, tostring(current_count + 1)}
"#;

#[derive(new, Clone)]
pub struct RedisCredentialsRepo {
    base_key: String,
    redis: ConnectionManager,
    max_sessions: usize,
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
        // Use a Vec to maintain order for TTL matching
        let mut session_entries = Vec::new();
        for (session_id_str, session_info_str) in sessions {
            let session_info: SessionInfo =
                serde_json::from_str(&session_info_str).map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to deserialize session info: {err}"
                    ))
                })?;

            // Parse session_id with proper error handling instead of unwrap
            let session_id = match Uuid::parse_str(&session_id_str) {
                Ok(id) => id,
                Err(err) => {
                    let _ = tracing::warn!(
                        "Invalid session_id format in Redis for user {user_id}: {session_id_str} - {err}"
                    );
                    // Skip this corrupted entry
                    continue;
                }
            };

            let expiry_key = self.get_expiry_key(user_id, &session_id);
            session_entries.push((session_id, session_info, expiry_key));
        }

        // Prepare TTL checks for all valid sessions
        for (_, _, expiry_key) in &session_entries {
            pipe.ttl(expiry_key);
        }

        // Execute batch TTL checks
        let ttls: Vec<i64> = if !session_entries.is_empty() {
            pipe.query_async(&mut self.redis.clone())
                .await
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to get batch TTLs: {err}"
                    ))
                })?
        } else {
            Vec::new()
        };

        // Apply TTLs to corresponding sessions
        for ((session_id, mut session_info, _), ttl) in
            session_entries.into_iter().zip(ttls.into_iter())
        {
            session_info.ttl_remaining = Some(ttl);
            result.insert(session_id, session_info);
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

        // Serialize session info
        let session_info_str =
            serde_json::to_string(session_info).map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to serialize session info: {err}"
                ))
            })?;

        // Use Lua script for atomic session creation
        let result: (i32, String) = redis::cmd("EVAL")
            .arg(CREATE_SESSION_LUA)
            .arg(1) // number of keys
            .arg(&key)
            .arg(&session_id_str)
            .arg(&session_info_str)
            .arg(&expiry_key)
            .arg(ttl_seconds)
            .arg(self.max_sessions)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to execute Lua script for session creation: {err}"
                ))
            })?;

        if result.0 == 0 {
            // Check if the error message indicates max sessions exceeded
            if result.1.contains("Maximum concurrent sessions") {
                return Err(DomainError::new_rate_limit_error(result.1));
            }
            return Err(DomainError::new_bad_input_error(result.1));
        }

        let new_count = result
            .1
            .parse::<i32>()
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to parse session count from Lua script: {err}"
                ))
            })?;

        let _ = tracing::info!("User has {new_count} sessions after creation");

        // Update active sessions metric for this user
        self.active_sessions
            .with_label_values(&[&user_id.to_string()])
            .set(new_count as f64);

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
        // Only extend TTL if the key exists and has a valid TTL (> -1)
        if ttl < 0 {
            return Err(DomainError::AuthError {
                message: format!("Session has expired or has no TTL - ttl: {ttl}"),
            });
        }

        // Calculate new TTL: add refresh_ttl_seconds to existing TTL
        // Ensure we don't create a negative TTL
        let existing_ttl = ttl;
        let refresh_ttl = self.refresh_ttl_seconds as i64;
        let new_ttl = existing_ttl + refresh_ttl;

        // Only update expiry if the new TTL is positive
        if new_ttl <= 0 {
            return Err(DomainError::AuthError {
                message: format!(
                    "Cannot extend session: new TTL would be non-positive (existing: {existing_ttl}, refresh: {refresh_ttl})"
                ),
            });
        }

        let _ = tracing::debug!(
            "Existing TTL for key {expiry_key}: {existing_ttl} seconds, extending to {new_ttl} seconds"
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
        let expiry_key = self.get_expiry_key(user_id, session_id);

        let mut pipe = redis::pipe();
        pipe.atomic()
            .hdel(&key, &session_id_str)
            .del(&expiry_key)
            .hlen(&key);

        let (_, _, count): ((), i32, i32) = pipe
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

        if session_ids.is_empty() {
            return Ok(());
        }

        // Build expiry keys and prepare for batch TTL check using mget
        let mut expiry_keys_with_idx = Vec::with_capacity(session_ids.len());
        for (idx, session_id_str) in session_ids.iter().enumerate() {
            // Parse session_id with proper error handling instead of unwrap
            let session_id = match Uuid::parse_str(session_id_str) {
                Ok(id) => id,
                Err(err) => {
                    let _ = tracing::warn!(
                        "Invalid session_id format in Redis for user {user_id}: {session_id_str} - {err}"
                    );
                    // Mark invalid session for deletion
                    continue;
                }
            };
            let expiry_key = self.get_expiry_key(user_id, &session_id);
            expiry_keys_with_idx.push((expiry_key, idx, session_id_str.clone()));
        }

        if expiry_keys_with_idx.is_empty() {
            return Ok(());
        }

        // Extract just the keys for mget
        let expiry_keys: Vec<&str> =
            expiry_keys_with_idx.iter().map(|(k, _, _)| k.as_str()).collect();

        // Batch check TTLs using mget - single round-trip instead of N
        let ttls: Vec<Option<i64>> = self
            .redis
            .clone()
            .mget(expiry_keys)
            .await
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to batch check TTLs for expired sessions: {err}"
                ))
            })?;

        // Build deletion pipeline with expired sessions
        let mut pipe = redis::pipe();
        let mut expired_count = 0;

        for ((_, _, session_id_str), ttl) in
            expiry_keys_with_idx.into_iter().zip(ttls.into_iter())
        {
            // TTL returns -2 if key doesn't exist (expired), -1 if no expiry, or remaining TTL in seconds
            if ttl == Some(-2) || ttl.is_none() {
                // Session has expired - delete from hash
                // Store session_id_str before moving it to hdel
                let session_id_str_copy = session_id_str.clone();
                pipe.hdel(&key, &session_id_str);

                // Also delete the expiry key to prevent memory leaks
                let session_id = Uuid::parse_str(&session_id_str_copy).unwrap_or_else(|_| {
                    // This should not happen as we already validated above
                    Uuid::nil()
                });
                let expiry_key = self.get_expiry_key(user_id, &session_id);
                pipe.del(expiry_key);
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

            let _ = tracing::info!(
                "Removed {expired_count} expired sessions, {count} active sessions remaining"
            );
        } else {
            let _ = tracing::debug!("No expired sessions found for user {user_id}");
        }

        Ok(())
    }
}
