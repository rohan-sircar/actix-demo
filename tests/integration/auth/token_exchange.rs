mod tests {
    use crate::common::{self, WithToken};
    use actix_demo::models::session::SessionInfo;
    use actix_http::{header, StatusCode};
    use serde_json::Value;

    #[actix_rt::test]
    async fn exchange_returns_token_and_user() {
        let ctx = common::TestContext::new(None).await;

        let mut resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": common::DEFAULT_USER
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = resp.json().await.unwrap();
        assert!(
            body.get("token").is_some(),
            "Response should contain 'token' field"
        );
        assert!(
            body.get("user").is_some(),
            "Response should contain 'user' field"
        );

        let user = body.get("user").unwrap();
        assert_eq!(
            user.get("username").and_then(|v| v.as_str()),
            Some(common::DEFAULT_USER),
            "Username should match"
        );
        assert!(
            user.get("email").is_some(),
            "Response should contain 'email' field"
        );
        assert!(
            user.get("id").is_some(),
            "Response should contain 'id' field"
        );

        let token = body.get("token").unwrap().as_str().unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
    }

    #[actix_rt::test]
    async fn exchange_rejects_wrong_password() {
        let ctx = common::TestContext::new(None).await;

        let resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": "wrong_password"
            }))
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 for wrong password"
        );
    }

    #[actix_rt::test]
    async fn exchange_rejects_nonexistent_user() {
        let ctx = common::TestContext::new(None).await;

        let resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": "nonexistent_user_12345",
                "password": "some_password"
            }))
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 for nonexistent user"
        );
    }

    #[actix_rt::test]
    async fn bearer_token_works_for_protected_endpoints() {
        let ctx = common::TestContext::new(None).await;

        // Get token via exchange endpoint
        let mut exchange_resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": common::DEFAULT_USER
            }))
            .await
            .unwrap();

        let body: Value = exchange_resp.json().await.unwrap();
        let token = body.get("token").unwrap().as_str().unwrap();

        // Use Bearer token for protected endpoint
        let resp = ctx
            .test_server
            .get("/api/v1/private/sessions")
            .with_bearer(token)
            .send()
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Bearer token should work for protected endpoints"
        );
    }

    #[actix_rt::test]
    async fn bearer_token_works_for_user_profile() {
        let ctx = common::TestContext::new(None).await;

        // Get token via exchange endpoint
        let mut exchange_resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": common::DEFAULT_USER
            }))
            .await
            .unwrap();

        let body: Value = exchange_resp.json().await.unwrap();
        let token = body.get("token").unwrap().as_str().unwrap();

        // Use Bearer token for user profile endpoint
        let resp = ctx
            .test_server
            .get("/api/v1/private/user")
            .with_bearer(token)
            .send()
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Bearer token should work for user profile"
        );
    }

    #[actix_rt::test]
    async fn missing_auth_header_returns_401() {
        let ctx = common::TestContext::new(None).await;

        let resp = ctx
            .test_server
            .get("/api/v1/private/sessions")
            .send()
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "Missing auth should return 401"
        );
    }

    #[actix_rt::test]
    async fn exchange_with_device_name_sets_device_name() {
        let ctx = common::TestContext::new(None).await;

        let mut resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": common::DEFAULT_USER,
                "device_name": "Test Mobile Device"
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = resp.json().await.unwrap();
        let token = body.get("token").unwrap().as_str().unwrap();

        // Check session has the device name via session headers
        let mut sessions_resp = ctx
            .test_server
            .get("/api/v1/private/sessions")
            .with_bearer(token)
            .send()
            .await
            .unwrap();

        assert_eq!(sessions_resp.status(), StatusCode::OK);

        let sessions: std::collections::HashMap<uuid::Uuid, SessionInfo> =
            sessions_resp.json().await.unwrap();
        assert!(!sessions.is_empty());

        // Find the session with the device name (there may be a previous session from TestContext init)
        let found = sessions
            .values()
            .any(|s| s.device_name.as_deref() == Some("Test Mobile Device"));
        assert!(
            found,
            "Expected a session with device_name 'Test Mobile Device'"
        );
    }

    #[actix_rt::test]
    async fn logout_works_with_bearer_token() {
        let ctx = common::TestContext::new(None).await;

        // Get token via exchange
        let mut exchange_resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": common::DEFAULT_USER
            }))
            .await
            .unwrap();

        let body: Value = exchange_resp.json().await.unwrap();
        let token = body.get("token").unwrap().as_str().unwrap();

        // Logout with Bearer token
        let resp = ctx
            .test_server
            .post("/api/v1/auth/logout")
            .with_bearer(token)
            .send()
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Logout should succeed with Bearer token"
        );

        // Verify token no longer works
        let resp = ctx
            .test_server
            .get("/api/v1/private/sessions")
            .with_bearer(token)
            .send()
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "Token should be invalid after logout"
        );
    }

    #[actix_rt::test]
    async fn revoke_other_sessions_works_with_bearer_token() {
        let ctx = common::TestContext::new(None).await;

        // Create multiple sessions via exchange
        let mut tokens = Vec::new();
        for i in 0..3 {
            let mut resp = ctx
                .test_server
                .post("/api/v1/auth/exchange")
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({
                    "username": common::DEFAULT_USER,
                    "password": common::DEFAULT_USER,
                    "device_name": format!("Device {i}")
                }))
                .await
                .unwrap();

            let body: Value = resp.json().await.unwrap();
            let token = body.get("token").unwrap().as_str().unwrap();
            tokens.push(token.to_string());
        }

        // Use first token to revoke others with Bearer
        let resp = ctx
            .test_server
            .post("/api/v1/private/sessions/revoke-others")
            .with_bearer(&tokens[0])
            .send()
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Revoke others should succeed with Bearer token"
        );

        // Verify first token still works
        let resp = ctx
            .test_server
            .get("/api/v1/private/sessions")
            .with_bearer(&tokens[0])
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        // Verify other tokens no longer work
        for token in &tokens[1..] {
            let resp = ctx
                .test_server
                .get("/api/v1/private/sessions")
                .with_bearer(token)
                .send()
                .await
                .unwrap();

            assert_eq!(
                resp.status(),
                StatusCode::UNAUTHORIZED,
                "Other sessions should be revoked"
            );
        }
    }

    #[actix_rt::test]
    async fn exchange_and_cookie_auth_both_create_sessions() {
        let ctx = common::TestContext::new(None).await;

        // Get token via exchange
        let mut exchange_resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": common::DEFAULT_USER
            }))
            .await
            .unwrap();

        let body: Value = exchange_resp.json().await.unwrap();
        let exchange_token = body.get("token").unwrap().as_str().unwrap();

        // Login via traditional login
        let cookie_token = common::get_http_token(
            &ctx.addr,
            common::DEFAULT_USER,
            common::DEFAULT_USER,
            &ctx.client,
        )
        .await
        .unwrap();

        // Both tokens should work
        let resp = ctx
            .test_server
            .get("/api/v1/private/sessions")
            .with_bearer(exchange_token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = ctx
            .test_server
            .get("/api/v1/private/sessions")
            .with_token(&cookie_token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn exchange_returns_consistent_user_data() {
        let ctx = common::TestContext::new(None).await;

        // Get token via exchange
        let mut exchange_resp = ctx
            .test_server
            .post("/api/v1/auth/exchange")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "username": common::DEFAULT_USER,
                "password": common::DEFAULT_USER
            }))
            .await
            .unwrap();

        let exchange_body: Value = exchange_resp.json().await.unwrap();
        let exchange_username = exchange_body
            .get("user")
            .unwrap()
            .get("username")
            .unwrap()
            .as_str()
            .unwrap();

        // Get token via traditional login and extract from cookie
        let cookie_token = common::get_http_token(
            &ctx.addr,
            common::DEFAULT_USER,
            common::DEFAULT_USER,
            &ctx.client,
        )
        .await
        .unwrap();

        // Use cookie token to get user profile
        let mut resp = ctx
            .test_server
            .get("/api/v1/private/user")
            .with_token(&cookie_token)
            .send()
            .await
            .unwrap();

        let profile_body: Value = resp.json().await.unwrap();
        let profile_username =
            profile_body.get("username").unwrap().as_str().unwrap();
        let profile_uuid = profile_body.get("user_uuid").unwrap();

        assert!(
            !profile_uuid.as_str().unwrap().is_empty(),
            "Profile should contain user_uuid"
        );
        assert_eq!(
            exchange_username, profile_username,
            "Username from exchange should match profile"
        );
    }
}
