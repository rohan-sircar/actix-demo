mod session_renewal;
mod sessions_api;

mod tests {
    use crate::common;
    use crate::common::TestContext;
    use actix_demo::utils;
    use actix_http::{header, StatusCode};
    use redis::AsyncCommands;
    use uuid::Uuid;

    pub async fn attempt_login(
        ctx: &TestContext,
        device_name: &str,
    ) -> (StatusCode, Option<String>) {
        let resp = ctx
            .test_server
            .post("/api/login")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": common::DEFAULT_USER,
                "device_name": device_name
            }))
            .await
            .unwrap();

        let status = resp.status();
        let token = if status == StatusCode::OK {
            Some(utils::extract_auth_token(resp.headers()).unwrap())
        } else {
            None
        };

        (status, token)
    }

    mod max_concurrent_sessions {
        use super::*;

        #[actix_rt::test]
        async fn should_limit_concurrent_sessions() {
            let mut ctx = TestContext::new(None).await;

            // Create 1 existing plus 4 new sessions successfully
            let _tokens =
                create_concurrent_sessions(&mut ctx, 4).await.unwrap();

            // Try 6th login which should be rejected
            let (status, _) = attempt_login(&ctx, "Test Device 6").await;

            assert_eq!(
                status,
                StatusCode::TOO_MANY_REQUESTS,
                "Expected 429 Too Many Requests for exceeding max sessions"
            );
        }

        pub async fn create_concurrent_sessions(
            ctx: &mut TestContext,
            count: usize,
        ) -> anyhow::Result<Vec<String>> {
            let mut tokens = Vec::new();

            for i in 0..count {
                let (status, token) =
                    attempt_login(ctx, &format!("Test Device {i}")).await;
                assert_eq!(
                    status,
                    StatusCode::OK,
                    "Expected successful login for attempt {}",
                    i + 1
                );
                tokens.push(token.unwrap());
            }

            Ok(tokens)
        }
    }

    mod race_condition {
        use super::*;

        // Integration tests for the race condition fix in session creation
        // The Lua script should ensure atomic session creation, preventing TOCTOU attacks

        #[actix_rt::test]
        async fn should_create_unique_session_for_each_login() {
            let ctx = TestContext::new(None).await;

            // Login once to create a valid session
            let (status1, token1) =
                attempt_login(&ctx, "Race Condition Test 1").await;
            assert_eq!(status1, StatusCode::OK, "First login should succeed");
            let token1 = token1.expect("First login should return a token");

            // Login again - should create a new unique session
            let (status2, token2) =
                attempt_login(&ctx, "Race Condition Test 2").await;
            assert_eq!(status2, StatusCode::OK, "Second login should succeed");
            let token2 = token2.expect("Second login should return a token");

            // Tokens should be different (new session)
            assert_ne!(
                token1, token2,
                "Each login should create a unique session token"
            );
        }

        #[actix_rt::test]
        async fn should_reject_duplicate_session_id() {
            // This test verifies that the Lua script prevents creating
            // a session with an ID that already exists for a user

            let ctx = TestContext::new(None).await;

            // Login to create initial session
            let (status, token) =
                attempt_login(&ctx, "Duplicate ID Test").await;
            assert_eq!(status, StatusCode::OK, "Initial login should succeed");
            let token = token.expect("Token should be present");

            // Parse token to get session_id
            let claims =
                actix_demo::utils::get_claims(&ctx.app_data.jwt_key, &token)
                    .expect("Should parse token claims");
            let user_id = claims.custom.user_id;
            let session_id = claims.custom.session_id;

            // Verify session exists
            let mut redis_conn = ctx.app_data.redis_conn_manager.clone();
            let session_key = ctx.app_data.credentials_repo.get_key(&user_id);

            let exists: bool = redis_conn
                .hexists(&session_key, session_id.to_string())
                .await
                .expect("Should check session existence");
            assert!(exists, "Session should exist in Redis");

            // Try to manually create a session with the same session_id
            // This should fail due to the Lua script's atomic check
            let now = chrono::Utc::now().naive_utc();
            let ttl_seconds = ctx.app_data.config.session.expiration_secs;

            let session_info = actix_demo::models::session::SessionInfo {
                session_id,
                device_id: Uuid::new_v4(),
                device_name: Some("Duplicate Test Device".to_string()),
                created_at: now,
                last_used_at: now,
                token: "duplicate_token".to_string(),
                ttl_remaining: Some(ttl_seconds as i64),
            };

            let result = ctx
                .app_data
                .credentials_repo
                .create_session(
                    &user_id,
                    &session_id,
                    &session_info,
                    ttl_seconds,
                )
                .await;

            // Should fail with "Session already exists" error
            assert!(result.is_err(), "Creating duplicate session should fail");

            let err_msg = format!("{:?}", result.unwrap_err());
            assert!(
                err_msg.contains("Session already exists"),
                "Error should indicate session already exists, got: {}",
                err_msg
            );
        }
    }

    mod session_cleanup {
        use super::*;

        // Integration tests for the session cleanup fix
        // Expiry keys should be deleted when sessions are removed

        #[actix_rt::test]
        async fn should_delete_expiry_key_on_session_removal() {
            let ctx = TestContext::new(None).await;

            // Login to create a session
            let (status, token) =
                attempt_login(&ctx, "Cleanup Test Device").await;
            assert_eq!(status, StatusCode::OK, "Expected successful login");
            let token = token.expect("Token should be present");

            // Parse token to get session details
            let claims =
                actix_demo::utils::get_claims(&ctx.app_data.jwt_key, &token)
                    .expect("Should parse token claims");
            let user_id = claims.custom.user_id;
            let session_id = claims.custom.session_id;

            // Get Redis connection to verify keys
            let mut redis_conn = ctx.app_data.redis_conn_manager.clone();

            // Verify the session key and expiry key exist
            let session_key = ctx.app_data.credentials_repo.get_key(&user_id);
            let expiry_key = ctx
                .app_data
                .credentials_repo
                .get_expiry_key(&user_id, &session_id);

            let session_exists: bool = redis_conn
                .hexists(&session_key, session_id.to_string())
                .await
                .expect("Should check session hash");
            let expiry_exists: bool = redis_conn
                .exists(&expiry_key)
                .await
                .expect("Should check expiry key");

            assert!(session_exists, "Session hash should exist");
            assert!(expiry_exists, "Expiry key should exist");

            // Logout to delete the session
            let resp = ctx
                .test_server
                .post("/api/logout")
                .append_header((
                    actix_http::header::CONTENT_TYPE,
                    "application/json",
                ))
                .insert_header(("Cookie", format!("X-AUTH-TOKEN={}", token)))
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK, "Logout should succeed");

            // Verify both session key and expiry key are deleted
            let session_after_delete: bool = redis_conn
                .hexists(&session_key, session_id.to_string())
                .await
                .expect("Should check session hash after delete");
            let expiry_after_delete: bool = redis_conn
                .exists(&expiry_key)
                .await
                .expect("Should check expiry key after delete");

            assert!(
                !session_after_delete,
                "Session hash should be deleted after logout"
            );
            assert!(
                !expiry_after_delete,
                "Expiry key should also be deleted after logout (no memory leak)"
            );
        }

        #[actix_rt::test]
        async fn should_cleanup_expiry_key_on_manual_revoke() {
            let ctx = TestContext::new(None).await;

            // Login to create a session
            let (status, token) =
                attempt_login(&ctx, "Revoke Test Device").await;
            assert_eq!(status, StatusCode::OK, "Login should succeed");
            let token = token.expect("Token should be present");

            // Parse token to get session details
            let claims =
                actix_demo::utils::get_claims(&ctx.app_data.jwt_key, &token)
                    .expect("Should parse token claims");
            let user_id = claims.custom.user_id;
            let session_id = claims.custom.session_id;

            // Get Redis connection to verify keys
            let mut redis_conn = ctx.app_data.redis_conn_manager.clone();
            let session_key = ctx.app_data.credentials_repo.get_key(&user_id);
            let expiry_key = ctx
                .app_data
                .credentials_repo
                .get_expiry_key(&user_id, &session_id);

            // Verify both keys exist
            let session_exists: bool = redis_conn
                .hexists(&session_key, session_id.to_string())
                .await
                .expect("Should check session");
            let expiry_exists: bool = redis_conn
                .exists(&expiry_key)
                .await
                .expect("Should check expiry");
            assert!(session_exists && expiry_exists, "Both keys should exist");

            // Revoke the session via API
            let resp = ctx
                .test_server
                .delete(&format!("/api/sessions/{}", session_id))
                .append_header((
                    actix_http::header::CONTENT_TYPE,
                    "application/json",
                ))
                .insert_header(("Cookie", format!("X-AUTH-TOKEN={}", token)))
                .send()
                .await
                .unwrap();

            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "Session revoke should succeed"
            );

            // Verify both keys are deleted
            let session_after_revoke: bool = redis_conn
                .hexists(&session_key, session_id.to_string())
                .await
                .expect("Should check session after revoke");
            let expiry_after_revoke: bool = redis_conn
                .exists(&expiry_key)
                .await
                .expect("Should check expiry after revoke");

            assert!(
                !session_after_revoke,
                "Session hash should be deleted after revoke"
            );
            assert!(
                !expiry_after_revoke,
                "Expiry key should also be deleted after revoke"
            );
        }
    }
}
