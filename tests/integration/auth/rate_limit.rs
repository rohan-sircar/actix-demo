mod tests {
    mod rate_limiting {
        use crate::common::{
            self, TestAppOptionsBuilder, TestContext, WithToken,
        };
        use actix_demo::models::rate_limit::RateLimitPolicy;
        use actix_http::StatusCode;
        use std::time::Duration;

        #[actix_rt::test]
        async fn should_rate_limit_api_requests() {
            let options = TestAppOptionsBuilder::default()
                .api_rate_limit(RateLimitPolicy {
                    max_requests: 2,
                    window_secs: 2,
                })
                .rate_limit_disabled(false)
                .build()
                .unwrap();

            let mut ctx = TestContext::new(Some(options)).await;
            let token = ctx.create_tokens(1).await.remove(0);

            // Send 2 valid requests
            for _ in 0..2 {
                let resp = ctx
                    .test_server
                    .get("/api/sessions")
                    .with_token(&token)
                    .send()
                    .await
                    .unwrap();

                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "Expected 200 OK for valid API request"
                );

                let headers = resp.headers();
                common::assert_rate_limit_headers(headers);
            }

            // Send 3rd request which should be rate limited
            let resp = ctx
                .test_server
                .get("/api/sessions")
                .with_token(&token)
                .send()
                .await
                .unwrap();

            let headers = resp.headers();
            common::assert_rate_limit_headers(headers);

            assert_eq!(
                resp.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Expected 429 Too Many Requests after rate limit exceeded"
            );

            // Test rate limit expiration
            let _ = tokio::time::sleep(Duration::from_secs(3)).await;

            // Try API request after window expiration
            let resp = ctx
                .test_server
                .get("/api/sessions")
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "Expected successful API request after rate limit window expired"
            );

            let headers = resp.headers();
            common::assert_rate_limit_headers(headers);
        }
    }
}
