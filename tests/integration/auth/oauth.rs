mod tests {
    use actix_demo::models::users::{Email, OAuthProvider, UserId};
    use actix_demo::schema::users;
    use actix_http::header::LOCATION;
    use actix_http::StatusCode;
    use diesel::prelude::*;
    use redis::AsyncCommands;
    use serde_json::Value;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::super::oauth_helpers::setup_oauth_app;
    use crate::common::TestContext;

    type UserRow = (
        UserId,
        actix_demo::models::users::Username,
        Email,
        Option<OAuthProvider>,
        Option<String>,
    );

    #[actix_rt::test]
    async fn test_github_login_redirects_with_state_and_challenge() {
        let mock_server = MockServer::start().await;

        let ctx = setup_oauth_app(&mock_server).await;

        let response = ctx
            .client
            .get(&format!("http://{}/api/v1/auth/oauth/github/login", ctx.addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);

        // Verify redirect location contains state and code_challenge parameters
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            location.contains("state="),
            "Redirect should contain state parameter"
        );
        assert!(
            location.contains("code_challenge="),
            "Redirect should contain code_challenge parameter"
        );
        assert!(
            location.contains("code_challenge_method=S256"),
            "Should use S256 challenge method"
        );

        // Extract state from redirect URL and verify it's stored in Redis
        let state = location
            .split("state=")
            .nth(1)
            .unwrap()
            .split('&')
            .next()
            .unwrap();
        let state_key = format!("app.oauth:state{}", state);
        let mut redis_conn = ctx.app_data.redis_conn_manager.clone();
        let code_verifier: Option<String> = redis::cmd("GET")
            .arg(&state_key)
            .query_async(&mut redis_conn)
            .await
            .unwrap();
        assert!(
            code_verifier.is_some(),
            "State should be stored in Redis with code verifier"
        );
    }

    #[actix_rt::test]
    async fn test_github_callback_happy_path() {
        let mock_server = MockServer::start().await;

        // Mock token exchange
        Mock::given(method("POST"))
            .and(path("/login/oauth/access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "access_token=gho_test123&scope=user%3Aemail&token_type=bearer",
            ))
            .mount(&mock_server)
            .await;

        // Mock user info
        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "id": 12345,
                    "login": "testuser",
                    "email": "testuser@example.com",
                    "name": "Test User",
                    "avatar_url": "https://example.com/avatar.png"
                }),
            ))
            .mount(&mock_server)
            .await;

        // Mock user emails
        Mock::given(method("GET"))
            .and(path("/user/emails"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"email": "testuser@example.com", "primary": true, "verified": true, "visibility": "public"}
            ])))
            .mount(&mock_server)
            .await;

        let ctx = setup_oauth_app(&mock_server).await;

        // Set state in Redis
        let mut redis = ctx.app_data.redis_conn_manager.clone();
        let state = "test-state";
        let code_verifier = "test-verifier";
        let key = format!("app.oauth:state{}", state);
        let _: () = redis.set_ex(&key, code_verifier, 300).await.unwrap();

        // Call callback
        let callback_url = format!(
            "http://{}/api/v1/auth/oauth/github/callback?code=test-auth-code&state={}",
            ctx.addr, state
        );
        let response = ctx.client.get(&callback_url).send().await.unwrap();

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);

        let cookies: Vec<_> =
            response.cookies().unwrap().iter().cloned().collect();
        assert!(
            cookies.iter().any(|c| c.name() == "X-AUTH-TOKEN"),
            "Session cookie should be set"
        );
    }

    #[actix_rt::test]
    async fn test_github_callback_invalid_state_returns_error() {
        let mock_server = MockServer::start().await;

        let ctx = setup_oauth_app(&mock_server).await;

        let mut response = ctx
            .test_server
            .get("/api/v1/auth/oauth/github/callback?code=test-code&state=invalid-state-xyz")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: Value = response.json().await.unwrap();
        assert!(body["cause"]
            .as_str()
            .unwrap()
            .contains("Session not found or expired"));
    }

    #[actix_rt::test]
    async fn test_github_callback_error_param_returns_json() {
        let mock_server = MockServer::start().await;

        let ctx = setup_oauth_app(&mock_server).await;

        let mut response = ctx
            .test_server
            .get("/api/v1/auth/oauth/github/callback?error=access_denied&error_description=User+cancelled+authorization")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: Value = response.json().await.unwrap();
        assert_eq!(body["error"], "access_denied");
    }

    #[actix_rt::test]
    async fn test_google_login_redirects_with_state_and_challenge() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/o/oauth2/v/auth"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let ctx = setup_oauth_app(&mock_server).await;

        let response = ctx
            .client
            .get(&format!("http://{}/api/v1/auth/oauth/google/login", ctx.addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);

        let location = response
            .headers()
            .get(LOCATION)
            .expect("Location header should be present");
        let location_str = location.to_str().unwrap();
        assert!(location_str.contains("/o/oauth2/v2/auth"));
        assert!(location_str.contains("client_id="));
        assert!(location_str.contains("redirect_uri="));
        assert!(location_str.contains("state="));
        assert!(location_str.contains("code_challenge="));
        assert!(location_str.contains("code_challenge_method=S256"));
        assert!(location_str.contains("scope=openid+email+profile"));
    }

    #[actix_rt::test]
    async fn test_google_callback_happy_path() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/oauth2/v4/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "access_token": "google_test_token",
                    "expires_in": 3600,
                    "token_type": "Bearer",
                    "scope": "openid email profile"
                }),
            ))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/oauth2/v2/userinfo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "sub": "google-12345",
                    "email": "googleuser@example.com",
                    "name": "Google User",
                    "email_verified": true,
                    "picture": "https://example.com/picture.png"
                }),
            ))
            .mount(&mock_server)
            .await;

        let ctx = setup_oauth_app(&mock_server).await;

        let mut redis = ctx.app_data.redis_conn_manager.clone();
        let state = "google-test-state";
        let code_verifier = "google-code-verifier";
        let key = format!("app.oauth:state{}", state);
        let _: () = redis.set_ex(&key, code_verifier, 300).await.unwrap();

        let callback_url = format!(
            "http://{}/api/v1/auth/oauth/google/callback?code=google-auth-code&state={}",
            ctx.addr, state
        );
        let response = ctx.client.get(&callback_url).send().await.unwrap();

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);

        let cookies: Vec<_> =
            response.cookies().unwrap().iter().cloned().collect();
        assert!(
            cookies.iter().any(|c| c.name() == "X-AUTH-TOKEN"),
            "Session cookie should be set"
        );

        let pool = &ctx.app_data.pool;
        let mut conn = pool.get().unwrap();
        let user = users::table
            .select((
                users::id,
                users::username,
                users::email,
                users::oauth_provider,
                users::oauth_uid,
            ))
            .order(users::id.desc())
            .first::<(
                UserId,
                actix_demo::models::users::Username,
                Email,
                Option<OAuthProvider>,
                Option<String>,
            )>(&mut conn)
            .unwrap();

        assert_eq!(user.2.as_str(), "googleuser@example.com");
        assert_eq!(user.3, Some(OAuthProvider::Google));
        assert_eq!(user.4, Some("google-12345".to_string()));
    }

    #[actix_rt::test]
    async fn test_account_linking_same_email_different_provider() {
        let mock_server = MockServer::start().await;

        // GitHub mocks
        Mock::given(method("POST"))
            .and(path("/login/oauth/access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "access_token=gho_linked&scope=user%3Aemail&token_type=bearer",
            ))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "id": 11111,
                    "login": "githubuser",
                    "email": "linked@example.com",
                    "name": "Linked User",
                    "avatar_url": "https://example.com/github.png"
                }),
            ))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user/emails"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!([
                    {
                        "email": "linked@example.com",
                        "primary": true,
                        "verified": true,
                        "visibility": "public"
                    }
                ]),
            ))
            .mount(&mock_server)
            .await;

        // Google mocks
        Mock::given(method("POST"))
            .and(path("/oauth2/v4/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "access_token": "google_linked_token",
                    "expires_in": 3600,
                    "token_type": "Bearer",
                    "scope": "openid email profile"
                }),
            ))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/oauth2/v2/userinfo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "sub": "google-99999",
                    "email": "linked@example.com",
                    "name": "Linked User Google",
                    "email_verified": true,
                    "picture": "https://example.com/google.png"
                }),
            ))
            .mount(&mock_server)
            .await;

        let github_ctx = setup_oauth_app(&mock_server).await;
        let mut redis = github_ctx.app_data.redis_conn_manager.clone();
        let _: () = redis
            .set_ex("app.oauth:stategithub-state", "github-verifier", 300)
            .await
            .unwrap();

        let github_response = github_ctx
            .client
            .get(&format!(
                "http://{}/api/v1/auth/oauth/github/callback?code=gh-code&state=github-state",
                github_ctx.addr
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(github_response.status(), StatusCode::TEMPORARY_REDIRECT);

        // Set Google state in the same Redis instance
        let mut redis = github_ctx.app_data.redis_conn_manager.clone();
        let _: () = redis
            .set_ex("app.oauth:stategoogle-state", "google-verifier", 300)
            .await
            .unwrap();

        // Google callback using the same context (shared DB)
        let google_response = github_ctx
            .client
            .get(&format!(
                "http://{}/api/v1/auth/oauth/google/callback?code=google-code&state=google-state",
                github_ctx.addr
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(google_response.status(), StatusCode::TEMPORARY_REDIRECT);

        let pool = &github_ctx.app_data.pool;
        let mut conn = pool.get().unwrap();

        let github_user: Option<UserRow> = users::table
            .select((
                users::id,
                users::username,
                users::email,
                users::oauth_provider,
                users::oauth_uid,
            ))
            .filter(users::email.eq("linked@example.com"))
            .order(users::id.desc())
            .first::<UserRow>(&mut conn)
            .ok();

        assert!(github_user.is_some());
        let user_id = github_user.unwrap().0.as_uint();

        let google_user: Option<UserRow> = users::table
            .select((
                users::id,
                users::username,
                users::email,
                users::oauth_provider,
                users::oauth_uid,
            ))
            .filter(users::oauth_provider.eq(OAuthProvider::Google))
            .filter(users::oauth_uid.eq("google-99999"))
            .first::<UserRow>(&mut conn)
            .ok();

        assert!(google_user.is_some());
        assert_eq!(google_user.unwrap().0.as_uint(), user_id);
    }

    #[actix_rt::test]
    async fn test_auto_registration_new_email() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/login/oauth/access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "access_token=gho_newuser&scope=user%3Aemail&token_type=bearer",
            ))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "id": 55555,
                    "login": "newgithubuser",
                    "email": "newuser@example.com",
                    "name": "New User",
                    "avatar_url": null
                }),
            ))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user/emails"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!([
                    {
                        "email": "newuser@example.com",
                        "primary": true,
                        "verified": true,
                        "visibility": null
                    }
                ]),
            ))
            .mount(&mock_server)
            .await;

        let ctx = setup_oauth_app(&mock_server).await;

        let mut redis = ctx.app_data.redis_conn_manager.clone();
        let _: () = redis
            .set_ex("app.oauth:statenew-user-state", "new-user-verifier", 300)
            .await
            .unwrap();

        let response = ctx
            .client
            .get(&format!(
                "http://{}/api/v1/auth/oauth/github/callback?code=new-code&state=new-user-state",
                ctx.addr
            ))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);

        let pool = &ctx.app_data.pool;
        let mut conn = pool.get().unwrap();
        let user = users::table
            .select((
                users::id,
                users::username,
                users::email,
                users::oauth_provider,
                users::oauth_uid,
            ))
            .filter(users::email.eq("newuser@example.com"))
            .first::<(
                UserId,
                actix_demo::models::users::Username,
                Email,
                Option<OAuthProvider>,
                Option<String>,
            )>(&mut conn)
            .unwrap();

        assert_eq!(user.2.as_str(), "newuser@example.com");
        assert_eq!(user.3, Some(OAuthProvider::Github));
        assert_eq!(user.4, Some("55555".to_string()));
    }

    #[actix_rt::test]
    async fn test_github_login_oauth_disabled() {
        let _mock_server = MockServer::start().await;

        let ctx = TestContext::new(None).await;

        let response = ctx
            .test_server
            .get("/api/v1/auth/oauth/github/login")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn test_google_login_oauth_disabled() {
        let _mock_server = MockServer::start().await;

        let ctx = TestContext::new(None).await;

        let response = ctx
            .test_server
            .get("/api/v1/auth/oauth/google/login")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn test_github_callback_missing_email_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/login/oauth/access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "access_token=gho_noemail&scope=user%3Aemail&token_type=bearer",
            ))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({
                    "id": 99999,
                    "login": "noemailuser",
                    "email": null,
                    "name": "No Email User",
                    "avatar_url": null
                }),
            ))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user/emails"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!([])),
            )
            .mount(&mock_server)
            .await;

        let ctx = setup_oauth_app(&mock_server).await;

        let mut redis = ctx.app_data.redis_conn_manager.clone();
        let _: () = redis
            .set_ex("app.oauth:stateno-email-state", "no-email-verifier", 300)
            .await
            .unwrap();

        let response = ctx
            .test_server
            .get("/api/v1/auth/oauth/github/callback?code=no-email-code&state=no-email-state")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_google_callback_invalid_state_returns_error() {
        let mock_server = MockServer::start().await;

        let ctx = setup_oauth_app(&mock_server).await;

        let mut response = ctx
            .test_server
            .get("/api/v1/auth/oauth/google/callback?code=google-code&state=invalid-google-state")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: Value = response.json().await.unwrap();
        assert!(body["cause"]
            .as_str()
            .unwrap()
            .contains("Session not found or expired"));
    }

    #[actix_rt::test]
    async fn test_google_callback_error_param_returns_json() {
        let mock_server = MockServer::start().await;

        let ctx = setup_oauth_app(&mock_server).await;

        let mut response = ctx
            .test_server
            .get("/api/v1/auth/oauth/google/callback?error=access_denied&error_description=User+cancelled")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: Value = response.json().await.unwrap();
        assert_eq!(body["error"], "access_denied");
    }
}
