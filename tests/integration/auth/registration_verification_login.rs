#[cfg(test)]
mod tests {
    use crate::common;
    use crate::common::WithToken;

    mod full_flow {
        use super::*;
        use crate::common::TestContext;
        use actix_http::header;
        use actix_web::http::StatusCode;
        use std::time::Duration;

        #[actix_rt::test]
        async fn should_complete_registration_verification_and_login() {
            let ctx = TestContext::new_with_mailpit(None).await;
            let addr = &ctx.addr;
            let client = &ctx.client;
            let mailpit = ctx.mailpit_client.as_ref().unwrap();

            // Cleanup any existing emails
            mailpit.delete_all().await.unwrap();

            let username = "flowtest_user";
            let password = "flowtest_pass123";
            let email = "flowtest@test.local";

            // Step 1: Register the user
            common::create_http_user_with_email(
                addr, username, password, email, client,
            )
            .await
            .unwrap();

            // Step 2: Wait for and extract verification token from email
            let token = mailpit
                .wait_for_verification_token(Duration::from_secs(5))
                .await
                .unwrap();

            // Step 3: Verify the email using the token
            let mut resp = ctx
                .test_server
                .post("/api/v1/auth/verify-email")
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({
                    "token": token
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: serde_json::Value = resp.json().await.unwrap();
            assert_eq!(body["success"], true);

            // Step 4: Login with the verified account
            let resp = ctx
                .test_server
                .post("/api/v1/auth/login")
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({
                    "username": username,
                    "password": password
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            // Step 5: Extract the auth token from response headers
            let auth_token =
                common::utils::extract_auth_token(resp.headers()).unwrap();
            assert!(!auth_token.is_empty(), "Expected X-AUTH-TOKEN to be set");

            // Step 6: Use the token to make a protected request
            let resp = ctx
                .test_server
                .get("/api/v1/sessions")
                .with_token(&auth_token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
        }

        #[actix_rt::test]
        async fn should_send_verification_email_on_registration() {
            let ctx = TestContext::new_with_mailpit(None).await;
            let addr = &ctx.addr;
            let client = &ctx.client;
            let mailpit = ctx.mailpit_client.as_ref().unwrap();

            mailpit.delete_all().await.unwrap();

            let username = "emailexample";
            let password = "pass123";
            let email = "emailtest@test.local";

            common::create_http_user_with_email(
                addr, username, password, email, client,
            )
            .await
            .unwrap();

            let msg = mailpit
                .wait_for_email(Duration::from_secs(5))
                .await
                .unwrap();

            assert_eq!(msg.to[0].address, email);
            assert!(msg.subject.contains("Verify"));
            assert!(msg.text.is_some(), "Email should have a text body");
            let text = msg.text.unwrap();
            assert!(
                text.contains("verify"),
                "Email text should mention verification"
            );
        }

        #[actix_rt::test]
        async fn should_fail_on_duplicate_registration() {
            let ctx = TestContext::new_with_mailpit(None).await;
            let addr = &ctx.addr;
            let client = &ctx.client;
            let mailpit = ctx.mailpit_client.as_ref().unwrap();

            mailpit.delete_all().await.unwrap();

            let username = "dupuser";
            let password = "pass123";
            let email = "dup@test.local";

            // First registration should succeed
            let resp = client
                .post(format!("http://{addr}/api/v1/auth/registration"))
                .insert_header(("content-type", "application/json"))
                .send_body(format!(
                    r#"{{"username":"{username}","password":"{password}","email":"{email}"}}"#
                ))
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);

            // Second registration with same email should fail
            let resp = client
                .post(format!("http://{addr}/api/v1/auth/registration"))
                .insert_header(("content-type", "application/json"))
                .send_body(format!(
                    r#"{{"username":"dupuser2","password":"{password}","email":"{email}"}}"#
                ))
                .await
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::BAD_REQUEST,
                "Duplicate email registration should return 400"
            );

            // Only one email should have been sent
            let msg = mailpit
                .wait_for_email(Duration::from_secs(5))
                .await
                .unwrap();
            assert_eq!(msg.to[0].address, email);

            // Delete the email and verify no more are sent
            mailpit.delete_all().await.unwrap();

            tokio::time::sleep(Duration::from_secs(2)).await;

            let resp = mailpit
                .http
                .get(format!("{}/api/v1/messages?limit=1", mailpit.base_url))
                .send()
                .await
                .unwrap()
                .json::<common::MessagesResponse>()
                .await
                .unwrap();

            assert!(
                resp.messages.is_empty(),
                "No additional emails should be sent after failed duplicate registration"
            );
        }

        #[actix_rt::test]
        async fn should_allow_login_without_email_verification() {
            // Documents current behavior: login does not check email verification status
            let ctx = TestContext::new_with_mailpit(None).await;
            let addr = &ctx.addr;
            let client = &ctx.client;
            let mailpit = ctx.mailpit_client.as_ref().unwrap();

            mailpit.delete_all().await.unwrap();

            let username = "unverified_user";
            let password = "unverified_pass123";
            let email = "unverified@test.local";

            // Register the user
            common::create_http_user_with_email(
                addr, username, password, email, client,
            )
            .await
            .unwrap();

            // Wait for the verification email to confirm it was sent
            let msg = mailpit
                .wait_for_email(Duration::from_secs(5))
                .await
                .unwrap();
            assert!(msg.subject.contains("Verify"));

            // Do NOT verify the email - attempt login anyway
            let resp = ctx
                .test_server
                .post("/api/v1/auth/login")
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({
                    "username": username,
                    "password": password
                }))
                .await
                .unwrap();

            // Current behavior: login succeeds even without email verification
            assert_eq!(resp.status(), StatusCode::OK);

            let token =
                common::utils::extract_auth_token(resp.headers()).unwrap();
            assert!(
                !token.is_empty(),
                "X-AUTH-TOKEN should be set even for unverified users"
            );
        }

        #[actix_rt::test]
        async fn should_reject_invalid_verification_token() {
            let ctx = TestContext::new_with_mailpit(None).await;

            // Try to verify with a non-existent token
            let mut resp = ctx
                .test_server
                .post("/api/v1/auth/verify-email")
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({
                    "token": "invalid_token_12345"
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: serde_json::Value = resp.json().await.unwrap();
            // Returns success but does nothing (prevents email enumeration)
            assert_eq!(body["success"], true);
        }

        #[actix_rt::test]
        async fn should_extract_token_from_email_correctly() {
            let ctx = TestContext::new_with_mailpit(None).await;
            let addr = &ctx.addr;
            let client = &ctx.client;
            let mailpit = ctx.mailpit_client.as_ref().unwrap();

            mailpit.delete_all().await.unwrap();

            let username = "tokenextract";
            let password = "pass123";
            let email = "tokenextract@test.local";

            common::create_http_user_with_email(
                addr, username, password, email, client,
            )
            .await
            .unwrap();

            let token = mailpit
                .wait_for_verification_token(Duration::from_secs(5))
                .await
                .unwrap();

            // Token should be a non-empty string (base64url encoded)
            assert!(!token.is_empty());
            assert!(token.len() > 10, "Token should be reasonably long");

            // The token should be usable for verification
            let mut resp = ctx
                .test_server
                .post("/api/v1/auth/verify-email")
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({
                    "token": token
                }))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: serde_json::Value = resp.json().await.unwrap();
            assert_eq!(body["success"], true);
        }
    }
}
