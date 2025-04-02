mod tests {
    use actix_demo::models::session::{
        SessionConfigBuilder, SessionRenewalPolicyBuilder,
    };
    use actix_http::{header::HeaderMap, StatusCode};

    use crate::common::{
        TestAppOptions, TestAppOptionsBuilder, TestContext, WithToken,
    };

    mod session_renewal {
        use crate::{
            auth::session::session_renewal::tests::test_request,
            common::{self, TestContext, WithToken},
        };
        use actix_http::StatusCode;
        use std::time::Duration;

        #[actix_rt::test]
        async fn should_expire_without_renewal() {
            // Set up test infrastructure with short session expiration
            let options = super::create_test_app_options_with_short_sessions();
            let mut ctx = TestContext::new(Some(options)).await;
            let token = ctx.create_tokens(1).await.pop().unwrap();

            // Initial request and validation
            let headers = test_request(&ctx, &token).await;
            let _ = common::assert_session_headers(&headers);

            // Verify TTL is within expected range (should be close to 2s)
            let ttl_remaining = common::get_ttl_remaining(&headers)
                .expect("Should have valid TTL remaining");
            assert!(
                ttl_remaining > 0 && ttl_remaining <= 2,
                "Initial TTL should be between 0 and 2 seconds, got {}",
                ttl_remaining
            );

            // Wait until after expiration (>4 seconds)
            let _ = tokio::time::sleep(Duration::from_secs(6)).await;

            // Verify expiration
            let resp = ctx
                .test_server
                .get("/api/sessions")
                .with_token(&token)
                .send()
                .await
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::UNAUTHORIZED,
                "Should expire after awaiting beyond TTL"
            );
        }

        #[actix_rt::test]
        async fn should_extend_ttl_on_renewal() {
            // Set up test infrastructure with short session expiration
            let options = super::create_test_app_options_with_short_sessions();
            let mut ctx = TestContext::new(Some(options)).await;
            let token = ctx.create_tokens(1).await.pop().unwrap();

            // Initial request and validation
            let headers = test_request(&ctx, &token).await;
            let _ = common::assert_session_headers(&headers);
            let initial_ttl = common::get_ttl_remaining(&headers)
                .expect("Should have valid TTL remaining");
            assert!(
                initial_ttl > 0 && initial_ttl <= 2,
                "Initial TTL should be between 0 and 2 seconds, got {}",
                initial_ttl
            );

            let initial_last_used = common::get_last_used_timestamp(&headers)
                .expect("Should have valid last used timestamp");
            let (session_id, device_id) =
                common::get_session_metadata(&headers)
                    .expect("Should have valid session metadata");

            // Wait 1s, then make request that should extend expiration by 2s
            // New expiration will be at t=3s (remaining 1s + 2s renewal)
            let _ = tokio::time::sleep(Duration::from_secs(1)).await;

            // First renewal request
            let headers = test_request(&ctx, &token).await;
            let _ = common::assert_session_headers(&headers);

            // Verify session metadata remains consistent
            let (renewed_session_id, renewed_device_id) =
                common::get_session_metadata(&headers)
                    .expect("Should have valid session metadata after renewal");
            assert_eq!(
                session_id, renewed_session_id,
                "Session ID should remain consistent after first renewal"
            );
            assert_eq!(
                device_id, renewed_device_id,
                "Device ID should remain consistent after first renewal"
            );

            // Verify TTL has been extended (should be close to 3s = remaining 1s + 2s renewal)
            let ttl_remaining = common::get_ttl_remaining(&headers)
                .expect("Should have valid TTL remaining after first renewal");
            assert!(
                ttl_remaining > 1 && ttl_remaining <= 3,
                "After first renewal, TTL should be between 1 and 3 seconds, got {}",
                ttl_remaining
            );

            // Verify last used timestamp has been updated
            let renewed_last_used = common::get_last_used_timestamp(&headers)
                .expect(
                    "Should have valid last used timestamp after first renewal",
                );
            assert_ne!(
                renewed_last_used, initial_last_used,
                "Last used timestamp should be updated after first renewal"
            );

            // Make second request that extends expiration again
            // Previous expiration was at t=3s, this will add 2s more
            let _ = tokio::time::sleep(Duration::from_secs(1)).await;

            // Second renewal request
            let headers = test_request(&ctx, &token).await;
            let _ = common::assert_session_headers(&headers);

            // Verify TTL has been extended again
            let ttl_remaining = common::get_ttl_remaining(&headers)
                .expect("Should have valid TTL remaining after second renewal");
            assert!(
                ttl_remaining > 3 && ttl_remaining <= 5,
                "After second renewal, TTL should be between 3 and 5 seconds, got {}",
                ttl_remaining
            );

            // Wait until after final expiration
            let _ = tokio::time::sleep(Duration::from_secs(6)).await;

            // Verify token has expired
            let resp = ctx
                .test_server
                .get("/api/sessions")
                .with_token(&token)
                .send()
                .await
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::UNAUTHORIZED,
                "Should expire after awaiting beyond renewed TTL"
            );
        }
    }

    pub fn create_test_app_options_with_short_sessions() -> TestAppOptions {
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

    async fn test_request(ctx: &TestContext, token: &str) -> HeaderMap {
        let resp = ctx
            .test_server
            .get("/api/sessions")
            .with_token(token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "Request should succeed");

        let headers = resp.headers().clone();

        headers
    }
}
