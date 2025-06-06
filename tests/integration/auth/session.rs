mod session_renewal;
mod sessions_api;

mod tests {
    mod max_concurrent_sessions {
        use crate::common::{self, TestContext};
        use actix_demo::utils;
        use actix_http::{header, StatusCode};

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
}
