mod rate_limit;
mod session;
use crate::common::{self, TestAppOptionsBuilder, TestContext};

#[cfg(test)]
mod tests {
    use crate::common::WithToken;

    use super::*;
    use actix_demo::{
        models::{rate_limit::RateLimitPolicy, users::UserId},
        workers,
    };
    use actix_http::{header, StatusCode};
    use tokio::{task::JoinHandle, time::sleep};

    use std::{str::FromStr, time::Duration};

    #[actix_rt::test]
    async fn should_rate_limit_failed_login_attempts() {
        // Create test context with custom rate limit policy
        let options = TestAppOptionsBuilder::default()
            .auth_rate_limit(RateLimitPolicy {
                max_requests: 5,
                window_secs: 2,
            })
            .rate_limit_disabled(false)
            .build()
            .unwrap();

        let ctx = TestContext::new(Some(options)).await;

        // Create test user
        let username = "test.user.1";
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";

        common::create_http_user(
            &ctx.addr,
            username,
            correct_password,
            &ctx.client,
        )
        .await
        .unwrap();

        // Send 5 failed login attempts
        for _ in 0..5 {
            let (status, headers) =
                login_attempt(&ctx, username, wrong_password).await;
            assert_eq!(
                status,
                StatusCode::UNAUTHORIZED,
                "Expected 401 Unauthorized for failed login attempt"
            );
            common::assert_rate_limit_headers(&headers);
        }

        // Send 6th login attempt which should be rate limited
        let (status, headers) =
            login_attempt(&ctx, username, wrong_password).await;
        common::assert_rate_limit_headers(&headers);
        assert_eq!(
            status,
            StatusCode::TOO_MANY_REQUESTS,
            "Expected 429 Too Many Requests after rate limit exceeded"
        );

        // Wait for rate limit window to expire
        sleep(Duration::from_secs(3)).await;

        // Try login with correct password after window expiration
        let (status, _) = login_attempt(&ctx, username, correct_password).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "Expected successful login after rate limit window expired"
        );
    }

    #[actix_rt::test]
    async fn should_expire_jwt_token_after_ttl() {
        use actix_demo::models::session::{
            SessionConfigBuilder, SessionRenewalPolicyBuilder,
        };

        // Create test context with custom session config
        let options = TestAppOptionsBuilder::default()
            .session_config(
                SessionConfigBuilder::default()
                    .expiration_secs(2)
                    .renewal(
                        SessionRenewalPolicyBuilder::default()
                            .renewal_window_secs(0)
                            .build()
                            .unwrap(),
                    )
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let ctx = TestContext::new(Some(options.clone())).await;

        // Get token
        let token = common::get_http_token(
            &ctx.addr,
            common::DEFAULT_USER,
            common::DEFAULT_USER,
            &ctx.client,
        )
        .await
        .unwrap();

        // Make valid request immediately
        let status = get_sessions(&ctx, &token).await;
        assert_eq!(status, StatusCode::OK, "Expected 200 OK for valid token");

        // Wait for token expiration
        sleep(Duration::from_secs(3)).await;

        // Make request with expired token
        let status = get_sessions(&ctx, &token).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "Expected 401 Unauthorized after token expiration"
        );
    }

    async fn login_attempt(
        ctx: &TestContext,
        username: &str,
        password: &str,
    ) -> (StatusCode, header::HeaderMap) {
        let resp = ctx
            .test_server
            .post("/api/login")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": username,
                "password": password
            }))
            .await
            .unwrap();

        let status = resp.status();
        let headers = resp.headers().clone();

        (status, headers)
    }

    async fn get_sessions(ctx: &TestContext, token: &str) -> StatusCode {
        let resp = ctx
            .test_server
            .get("/api/sessions")
            .with_token(token)
            .send()
            .await
            .unwrap();

        resp.status()
    }

    #[actix_rt::test]
    async fn should_cleanup_expired_sessions() {
        use actix_demo::models::session::{
            SessionConfigBuilder, SessionRenewalPolicyBuilder,
        };

        // Create test context with custom session config
        let options = TestAppOptionsBuilder::default()
            .session_config(
                SessionConfigBuilder::default()
                    .expiration_secs(2)
                    .renewal(
                        SessionRenewalPolicyBuilder::default()
                            .renewal_window_secs(0)
                            .build()
                            .unwrap(),
                    )
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let ctx = TestContext::new(Some(options.clone())).await;

        let _sessions_cleanup_worker_handle: JoinHandle<()> = {
            let config = options.sessions_cleanup_worker_config;
            workers::start_sessions_cleanup_worker(
                config,
                ctx.app_data.credentials_repo.clone(),
                ctx.app_data.user_ids_cache.clone(),
                ctx.app_data.pool.clone(),
            )
            .await
        };

        // Perform login to create a session
        let _token = common::get_http_token(
            &ctx.addr,
            common::DEFAULT_USER,
            common::DEFAULT_USER,
            &ctx.client,
        )
        .await
        .unwrap();

        // Verify session exists
        let user_id = UserId::from_str("1").unwrap();
        let sessions = ctx
            .app_data
            .credentials_repo
            .load_all_sessions(&user_id)
            .await
            .unwrap();
        assert!(
            !sessions.is_empty(),
            "Expected at least one session after login"
        );

        // Wait for session expiration and cleanup
        sleep(Duration::from_secs(6)).await;

        // Verify session was cleaned up
        let sessions = ctx
            .app_data
            .credentials_repo
            .load_all_sessions(&user_id)
            .await
            .unwrap();
        assert!(sessions.is_empty(), "Expected no sessions after cleanup");
    }
}
