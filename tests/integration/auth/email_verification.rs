#[cfg(test)]
mod tests {
    use crate::common;

    mod verify_email_api {
        use crate::common::TestContext;
        use actix_web::http::StatusCode;

        #[actix_rt::test]
        async fn should_return_success_for_unverified_token() {
            let ctx = TestContext::new(None).await;

            let mut resp = ctx
                .test_server
                .post("/api/email/verify")
                .append_header(("content-type", "application/json"))
                .send_json(&serde_json::json!({
                    "token": "nonexistent_token_12345"
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: serde_json::Value = resp.json().await.unwrap();
            assert_eq!(body["success"], true);
            assert_eq!(
                body["message"].as_str().unwrap(),
                "If the email is registered, you will receive a verification email"
            );
        }

        #[actix_rt::test]
        async fn should_return_success_for_random_token() {
            let ctx = TestContext::new(None).await;

            let mut resp = ctx
                .test_server
                .post("/api/email/verify")
                .append_header(("content-type", "application/json"))
                .send_json(&serde_json::json!({
                    "token": "some_random_token"
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: serde_json::Value = resp.json().await.unwrap();
            assert_eq!(body["success"], true);
        }
    }

    mod password_reset_api {
        use crate::common::TestContext;
        use actix_web::http::StatusCode;

        use super::*;

        #[actix_rt::test]
        async fn should_return_success_for_registered_email() {
            let ctx = TestContext::new(None).await;
            let addr = &ctx.addr;
            let client = &ctx.client;

            common::create_http_user_with_email(
                addr,
                "reset_user",
                "testpass123",
                "reset_user@test.local",
                client,
            )
            .await
            .unwrap();

            let mut resp = ctx
                .test_server
                .post("/api/password-reset/request")
                .append_header(("content-type", "application/json"))
                .send_json(&serde_json::json!({
                    "email": "reset_user@test.local"
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: serde_json::Value = resp.json().await.unwrap();
            assert_eq!(body["success"], true);
            assert_eq!(
                body["message"].as_str().unwrap(),
                "If the email is registered, you will receive a password reset link"
            );
        }

        #[actix_rt::test]
        async fn should_return_success_for_unregistered_email() {
            let ctx = TestContext::new(None).await;

            let mut resp = ctx
                .test_server
                .post("/api/password-reset/request")
                .append_header(("content-type", "application/json"))
                .send_json(&serde_json::json!({
                    "email": "nonexistent@test.local"
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: serde_json::Value = resp.json().await.unwrap();
            assert_eq!(body["success"], true);
        }

        #[actix_rt::test]
        async fn should_return_success_for_invalid_reset_token() {
            let ctx = TestContext::new(None).await;

            // Send a reset request first to ensure the token table has entries
            let _ = ctx
                .test_server
                .post("/api/password-reset/request")
                .append_header(("content-type", "application/json"))
                .send_json(&serde_json::json!({
                    "email": "admin@example.com"
                }))
                .await
                .unwrap();

            // Test with invalid token - should return success (prevents email enumeration)
            let mut resp = ctx
                .test_server
                .post("/api/password-reset/complete")
                .append_header(("content-type", "application/json"))
                .send_json(&serde_json::json!({
                    "token": "nonexistent_reset_token",
                    "new_password": "newpassword123"
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: serde_json::Value = resp.json().await.unwrap();
            assert_eq!(body["success"], true);
        }

        #[actix_rt::test]
        async fn should_return_success_for_expired_token_pattern() {
            let ctx = TestContext::new(None).await;

            let resp = ctx
                .test_server
                .post("/api/password-reset/complete")
                .append_header(("content-type", "application/json"))
                .send_json(&serde_json::json!({
                    "token": "expired_token_abc123",
                    "new_password": "newpassword456"
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}
