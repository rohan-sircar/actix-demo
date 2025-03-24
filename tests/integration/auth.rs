use crate::common;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_http::{header, StatusCode};
    use anyhow::anyhow;
    use awc::Client;

    use std::time::Duration;

    mod login_rate_limiting {
        use actix_demo::models::rate_limit::RateLimitPolicy;

        use crate::common::TestAppOptionsBuilder;

        use super::*;

        #[actix_rt::test]
        async fn should_rate_limit_failed_login_attempts() {
            let res: anyhow::Result<()> = async {
                // Set up test infrastructure
                let (pg_connstr, _pg) =
                    common::test_with_postgres().await.unwrap();
                let (redis_connstr, _redis) = common::test_with_redis().await?;

                // Create test app instance
                let test_server = common::test_http_app(
                    &pg_connstr,
                    &redis_connstr,
                    TestAppOptionsBuilder::default()
                        .auth_rate_limit(RateLimitPolicy {
                            max_requests: 5,
                            window_secs: 2,
                        })
                        .rate_limit_disabled(false)
                        .build()
                        .unwrap(),
                )
                .await?;

                let addr = test_server.addr().to_string();
                let client = Client::new();

                // Create test user
                let username = "test.user.1";
                let correct_password = "correct_password";

                common::create_http_user(
                    &addr,
                    username,
                    correct_password,
                    &client,
                )
                .await?;

                // Send 5 failed login attempts
                let wrong_password = "wrong_password";
                for _ in 0..5 {
                    let resp = client
                        .post(format!("http://{addr}/api/login"))
                        .append_header((
                            header::CONTENT_TYPE,
                            "application/json",
                        ))
                        .send_json(&serde_json::json!({
                            "username": username,
                            "password": wrong_password
                        }))
                        .await
                        .map_err(|err| anyhow!("{err}"))?;

                    assert_eq!(
                        resp.status(),
                        StatusCode::UNAUTHORIZED,
                        "Expected 401 Unauthorized for failed login attempt"
                    );

                    let headers = resp.headers();

                    common::assert_rate_limit_headers(headers);
                }

                // Send 6th login attempt which should be rate limited
                let resp = client
                    .post(format!("http://{addr}/api/login"))
                    .append_header((header::CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "username": username,
                        "password": wrong_password
                    }))
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                let headers = resp.headers();

                common::assert_rate_limit_headers(headers);

                assert_eq!(
                    resp.status(),
                    StatusCode::TOO_MANY_REQUESTS,
                    "Expected 429 Too Many Requests after rate limit exceeded"
                );

                // Optional: Test rate limit expiration
                // Sleep for window_secs + 1 seconds to allow rate limit window to expire
                let _ = tokio::time::sleep(Duration::from_secs(3)).await;

                // Try login with correct password after window expiration
                let resp = client
                    .post(format!("http://{addr}/api/login"))
                    .append_header((header::CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "username": username,
                        "password": correct_password
                    }))
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "Expected successful login after rate limit window expired"
                );

                Ok(())
            }
            .await;

            tracing::info!("{res:?}");
            res.unwrap();
        }
    }

    mod token_expiration {
        use crate::common::{TestAppOptionsBuilder, WithToken};

        use super::*;
        use actix_demo::models::session::{
            SessionConfigBuilder, SessionRenewalPolicyBuilder,
        };
        use actix_http::StatusCode;
        use anyhow::anyhow;
        use awc::Client;
        use std::time::Duration;

        #[actix_rt::test]
        async fn should_expire_jwt_token_after_ttl() {
            let res: anyhow::Result<()> = async {
                // Set up test infrastructure with 5-second token expiration
                let (pg_connstr, _pg) = common::test_with_postgres().await?;
                let (redis_connstr, _redis) = common::test_with_redis().await?;

                // Create test app with session expiration configuration
                let test_server = common::test_http_app(
                    &pg_connstr,
                    &redis_connstr,
                    TestAppOptionsBuilder::default()
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
                        .unwrap(),
                )
                .await?;

                let addr = test_server.addr().to_string();
                let client = Client::new();

                // Create test user
                let username = "ttl.test.user";
                let password = "test_password";
                let _ = common::create_http_user(
                    &addr, username, password, &client,
                )
                .await?;

                // Login to get token
                let token =
                    common::get_http_token(&addr, username, password, &client)
                        .await?;

                // Make valid request immediately
                let resp = client
                    .get(format!("http://{addr}/api/users?page=0&limit=5"))
                    .with_token(&token)
                    .send()
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "Expected 200 OK for valid token"
                );

                // Wait for token expiration
                let _ = tokio::time::sleep(Duration::from_secs(3)).await;

                // Make request with expired token
                let resp = client
                    .get(format!("http://{addr}/api/users?page=0&limit=5"))
                    .with_token(&token)
                    .send()
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                assert_eq!(
                    resp.status(),
                    StatusCode::UNAUTHORIZED,
                    "Expected 401 Unauthorized after token expiration"
                );
                Ok(())
            }
            .await;
            tracing::info!("{res:?}");
            res.unwrap();
        }
    }

    mod max_concurrent_sessions {
        use crate::common::TestAppOptionsBuilder;

        use super::*;
        use actix_http::StatusCode;
        use anyhow::anyhow;
        use awc::Client;

        #[actix_rt::test]
        async fn should_limit_concurrent_sessions() {
            let res: anyhow::Result<()> = async {
                // Set up test infrastructure
                let (pg_connstr, _pg) = common::test_with_postgres().await?;
                let (redis_connstr, _redis) = common::test_with_redis().await?;

                // Create test app instance
                let test_server = common::test_http_app(
                    &pg_connstr,
                    &redis_connstr,
                    TestAppOptionsBuilder::default().build().unwrap(),
                )
                .await?;

                let addr = test_server.addr().to_string();
                let client = Client::new();

                // Create test user
                let username = "session.test.user";
                let password = "test_password";

                common::create_http_user(&addr, username, password, &client)
                    .await?;

                // Perform 5 successful logins
                for i in 0..5 {
                    let resp = client
                        .post(format!("http://{addr}/api/login"))
                        .append_header((
                            header::CONTENT_TYPE,
                            "application/json",
                        ))
                        .send_json(&serde_json::json!({
                            "username": username,
                            "password": password,
                            // "device_id": format!("device_{i}"),
                            "device_name": format!("Test Device {i}")
                        }))
                        .await
                        .map_err(|err| anyhow!("{err}"))?;

                    assert_eq!(
                        resp.status(),
                        StatusCode::OK,
                        "Expected successful login for attempt {}",
                        i + 1
                    );
                }

                // Try 6th login which should be rejected
                let resp = client
                    .post(format!("http://{addr}/api/login"))
                    .append_header((header::CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "username": username,
                        "password": password,
                        // "device_id": "device_6",
                        "device_name": "Test Device 6"
                    }))
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                assert_eq!(
                    resp.status(),
                    StatusCode::TOO_MANY_REQUESTS,
                    "Expected 429 Too Many Requests for exceeding max sessions"
                );

                Ok(())
            }
            .await;

            tracing::info!("{res:?}");
            res.unwrap();
        }
    }
    mod session_renewal {
        use super::*;
        use crate::common::{TestAppOptions, TestAppOptionsBuilder, WithToken};
        use actix_demo::models::session::{
            SessionConfigBuilder, SessionRenewalPolicyBuilder,
        };
        use actix_http::{header::HeaderMap, StatusCode};
        use anyhow::anyhow;
        use awc::Client;
        use std::time::Duration;

        fn create_test_app_options_with_short_sessions() -> TestAppOptions {
            TestAppOptionsBuilder::default()
                .session_config(
                    SessionConfigBuilder::default()
                        // Sessions expire after 2 seconds by default
                        .expiration_secs(2)
                        .renewal(
                            SessionRenewalPolicyBuilder::default()
                                // Each successful request extends expiration by 2 seconds
                                // New expiration = remaining_time + renewal_window_secs
                                .renewal_window_secs(2)
                                .build()
                                .unwrap(),
                        )
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap()
        }

        #[actix_rt::test]
        async fn should_expire_without_renewal() {
            let res: anyhow::Result<()> = async {
                // Set up test infrastructure with session renewal configuration
                let (pg_connstr, _pg) = common::test_with_postgres().await?;
                let (redis_connstr, _redis) = common::test_with_redis().await?;

                // Create test app with session renewal policy
                let test_server = common::test_http_app(
                    &pg_connstr,
                    &redis_connstr,
                    create_test_app_options_with_short_sessions(),
                )
                .await?;

                let addr = test_server.addr().to_string();
                let client = Client::new();

                // Create test user
                let username = "renewal.test.user";
                let password = "test_password";
                common::create_http_user(&addr, username, password, &client)
                    .await?;

                //// first test that token expires at >4 seconds because of no renewal ////
                // Login to get initial token
                let token =
                    common::get_http_token(&addr, username, password, &client)
                        .await?;

                // Initial valid request
                let resp = client
                    .get(format!("http://{addr}/api/users?page=0&limit=5"))
                    .with_token(&token)
                    .send()
                    .await
                    .map_err(|err| anyhow!("{err}"))?;
                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "Initial request should succeed"
                );

                // Verify session headers after initial request
                let headers = resp.headers();
                assert_session_headers(headers);

                // Verify TTL is within expected range (should be close to 2s)
                let ttl_remaining = get_ttl_remaining(headers)
                    .expect("Should have valid TTL remaining");
                assert!(
                    ttl_remaining > 0 && ttl_remaining <= 2,
                    "Initial TTL should be between 0 and 2 seconds, got {}",
                    ttl_remaining
                );

                let _initial_last_used = headers
                    .get("x-session-last-used-at")
                    .and_then(|v| v.to_str().ok())
                    .expect("Should have valid last used timestamp");

                // Store session metadata for consistency checks
                let _session_id = headers
                    .get("x-session-id")
                    .and_then(|v| v.to_str().ok())
                    .expect("Should have valid session ID");
                let _device_id = headers
                    .get("x-session-device-id")
                    .and_then(|v| v.to_str().ok())
                    .expect("Should have valid device ID");

                // Wait until renewal window (2 seconds passed, 3 seconds remaining)
                let _ = tokio::time::sleep(Duration::from_secs(6)).await;

                // Verify expiration
                let resp = client
                    .get(format!("http://{addr}/api/users?page=0&limit=5"))
                    .with_token(&token)
                    .send()
                    .await
                    .map_err(|err| anyhow!("{err}"))?;
                assert_eq!(
                    resp.status(),
                    StatusCode::UNAUTHORIZED,
                    "Should expire after awaiting beyond TTL"
                );
                Ok(())
            }
            .await;
            tracing::info!("{res:?}");
            res.unwrap();
        }

        #[actix_rt::test]
        async fn should_extend_ttl_on_renewal() {
            // Set up test infrastructure with session renewal configuration
            let (pg_connstr, _pg) = common::test_with_postgres().await.unwrap();
            let (redis_connstr, _redis) =
                common::test_with_redis().await.unwrap();

            // Create test app with session renewal policy
            let test_server = common::test_http_app(
                &pg_connstr,
                &redis_connstr,
                create_test_app_options_with_short_sessions(),
            )
            .await
            .unwrap();

            let addr = test_server.addr().to_string();
            let client = Client::new();

            // Create test user
            let username = "renewal.test.user";
            let password = "test_password";

            let _ =
                common::create_http_user(&addr, username, password, &client)
                    .await
                    .unwrap();

            // Login to get initial token
            let token =
                common::get_http_token(&addr, username, password, &client)
                    .await
                    .unwrap();

            // Initial request should succeed
            let resp = client
                .get(format!("http://{addr}/api/users?page=0&limit=5"))
                .with_token(&token)
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "Initial request should succeed"
            );

            // Verify session headers are present with initial values
            let headers = resp.headers();
            assert_session_headers(headers);

            // Verify TTL is within expected range (should be close to 2s)
            let ttl_remaining = get_ttl_remaining(headers)
                .expect("Should have valid TTL remaining");
            assert!(
                ttl_remaining > 0 && ttl_remaining <= 2,
                "Initial TTL should be between 0 and 2 seconds, got {}",
                ttl_remaining
            );

            let initial_last_used = headers
                .get("x-session-last-used-at")
                .and_then(|v| v.to_str().ok())
                .expect("Should have valid last used timestamp");

            // Store session metadata for consistency checks
            let session_id = headers
                .get("x-session-id")
                .and_then(|v| v.to_str().ok())
                .expect("Should have valid session ID");
            let device_id = headers
                .get("x-session-device-id")
                .and_then(|v| v.to_str().ok())
                .expect("Should have valid device ID");

            // t=0: Initial token with 2s expiration
            // Wait 1s, then make request that should extend expiration by 2s
            // New expiration will be at t=3s (remaining 1s + 2s renewal)
            let _ = tokio::time::sleep(Duration::from_secs(1)).await;

            // Renewal request should succeed
            let resp = client
                .get(format!("http://{addr}/api/users?page=0&limit=5"))
                .with_token(&token)
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "First renewal request should succeed"
            );

            // Verify session headers after first renewal
            let headers = resp.headers();
            assert_session_headers(headers);

            // Verify session metadata remains consistent
            assert_eq!(
                headers
                    .get("x-session-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap(),
                session_id,
                "Session ID should remain consistent after first renewal"
            );
            assert_eq!(
                headers
                    .get("x-session-device-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap(),
                device_id,
                "Device ID should remain consistent after first renewal"
            );

            // Verify TTL has been extended (should be close to 3s = remaining 1s + 2s renewal)
            let ttl_remaining = get_ttl_remaining(headers)
                .expect("Should have valid TTL remaining after first renewal");
            assert!(ttl_remaining > 1 && ttl_remaining <= 3,
                    "After first renewal, TTL should be between 1 and 3 seconds, got {}", ttl_remaining);

            // Verify last used timestamp has been updated
            let renewed_last_used = headers
                .get("x-session-last-used-at")
                .and_then(|v| v.to_str().ok())
                .expect(
                    "Should have valid last used timestamp after first renewal",
                );
            assert_ne!(
                renewed_last_used, initial_last_used,
                "Last used timestamp should be updated after first renewal"
            );

            // t=2s: Make second request that extends expiration again
            // Previous expiration was at t=3s, this will add 2s more
            // New expiration will be at t=5s (remaining 1s + 2s renewal)
            let _ = tokio::time::sleep(Duration::from_secs(1)).await;

            // Verify token is still valid and will be renewed again
            // Current time t=2s, still within renewed expiration (t=5s)
            // This request will extend expiration to t=7s (remaining 3s + 2s renewal)
            let resp = client
                .get(format!("http://{addr}/api/users?page=0&limit=5"))
                .with_token(&token)
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "Second renewal request should succeed"
            );

            // Verify session headers after second renewal
            let headers = resp.headers();
            assert_session_headers(headers);

            // Verify session metadata remains consistent
            assert_eq!(
                headers
                    .get("x-session-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap(),
                session_id,
                "Session ID should remain consistent after second renewal"
            );
            assert_eq!(
                headers
                    .get("x-session-device-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap(),
                device_id,
                "Device ID should remain consistent after second renewal"
            );

            // Verify TTL has been extended (should be close to 5s = remaining 3s + 2s renewal)
            let ttl_remaining = get_ttl_remaining(headers)
                .expect("Should have valid TTL remaining after second renewal");
            assert!(ttl_remaining > 3 && ttl_remaining <= 5,
                    "After second renewal, TTL should be between 3 and 5 seconds, got {}", ttl_remaining);

            // Verify last used timestamp has been updated again
            let second_renewed_last_used = headers
                    .get("x-session-last-used-at")
                    .and_then(|v| v.to_str().ok())
                    .expect("Should have valid last used timestamp after second renewal");
            assert_ne!(
                second_renewed_last_used, renewed_last_used,
                "Last used timestamp should be updated after second renewal"
            );

            // Wait until after final expiration
            // Current time t=2s, final expiration at t=7s
            // Wait 6s to ensure we're past expiration (t=8s)
            let _ = tokio::time::sleep(Duration::from_secs(6)).await;

            // Verify token has expired
            // Current time t=8s, which is past final expiration at t=7s
            // Timeline of events:
            // t=0s: Initial token (exp t=2s)
            // t=1s: First renewal (exp t=3s)
            // t=2s: Second renewal (exp t=5s)
            // t=2s: Third renewal (exp t=7s)
            // t=8s: Current time (token expired)
            let resp = client
                .get(format!("http://{addr}/api/users?page=0&limit=5"))
                .with_token(&token)
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::UNAUTHORIZED,
                "Should expire after awaiting beyond renewed TTL"
            );
        }

        fn assert_session_headers(headers: &HeaderMap) {
            // Verify session headers are present with initial values
            assert!(
                headers.contains_key("x-session-id"),
                "Missing session ID header"
            );
            assert!(
                headers.contains_key("x-session-device-id"),
                "Missing device ID header"
            );
            assert!(
                headers.contains_key("x-session-created-at"),
                "Missing created at header"
            );
            assert!(
                headers.contains_key("x-session-last-used-at"),
                "Missing last used header"
            );
            assert!(
                headers.contains_key("x-session-ttl-remaining"),
                "Missing TTL remaining header"
            );
        }

        fn get_ttl_remaining(headers: &HeaderMap) -> Option<i64> {
            headers
                .get("x-session-ttl-remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok())
        }
    }
}
