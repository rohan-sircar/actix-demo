#[cfg(test)]
mod tests {
    use actix_http::header;
    use actix_web::http::StatusCode;

    mod admin_route_access {
        use crate::common;

        use crate::common::{get_http_token, TestContext, WithToken};

        use super::*;

        /// Verifies that non-admin users (RoleUser) get 403 on admin-only routes.
        /// Registration via /api/registration assigns RoleUser by default
        /// (insert_new_regular_user → insert_new_user with RoleEnum::RoleUser).
        #[actix_rt::test]
        async fn should_return_403_for_non_admin_on_post_cmd() {
            let ctx = TestContext::new(None).await;

            // Register a regular user (RoleUser)
            let _ = common::create_http_user(
                &ctx.addr,
                "regularuser",
                "testpass",
                &ctx.client,
            )
            .await;

            // Login as regular user to get token
            let token = get_http_token(
                &ctx.addr,
                "regularuser",
                "testpass",
                &ctx.client,
            )
            .await
            .unwrap();

            // Try to POST /api/cmd (admin-only)
            let resp = ctx
                .test_server
                .post("/api/cmd")
                .with_token(&token)
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({"args": ["test"]}))
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }

        #[actix_rt::test]
        async fn should_return_403_for_non_admin_on_get_job() {
            use uuid::Uuid;

            let ctx = TestContext::new(None).await;

            // Register a regular user
            let _ = common::create_http_user(
                &ctx.addr,
                "regularuser2",
                "testpass",
                &ctx.client,
            )
            .await;

            // Login as regular user
            let token = get_http_token(
                &ctx.addr,
                "regularuser2",
                "testpass",
                &ctx.client,
            )
            .await
            .unwrap();

            // Try to GET /api/cmd/{job_id} (admin-only)
            let fake_job_id = Uuid::new_v4().to_string();
            let resp = ctx
                .test_server
                .get(format!("/api/cmd/{}", fake_job_id))
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }

        #[actix_rt::test]
        async fn should_return_403_for_non_admin_on_delete_job() {
            use uuid::Uuid;

            let ctx = TestContext::new(None).await;

            // Register a regular user
            let _ = common::create_http_user(
                &ctx.addr,
                "regularuser3",
                "testpass",
                &ctx.client,
            )
            .await;

            // Login as regular user
            let token = get_http_token(
                &ctx.addr,
                "regularuser3",
                "testpass",
                &ctx.client,
            )
            .await
            .unwrap();

            // Try to DELETE /api/cmd/{job_id} (admin-only)
            let fake_job_id = Uuid::new_v4().to_string();
            let resp = ctx
                .test_server
                .delete(format!("/api/cmd/{}", fake_job_id))
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }

        /// Verifies that the default test user (RoleAdmin, created at common/mod.rs:539)
        /// is NOT blocked by the role gate on admin routes.
        /// Note: assert_ne is intentional — this tests the role gate passes,
        /// not that the handler succeeds (which may fail for other reasons
        /// like missing bin, invalid args, etc.).
        #[actix_rt::test]
        async fn should_allow_admin_on_admin_routes() {
            use uuid::Uuid;

            let ctx = TestContext::new(None).await;

            // DEFAULT_USER is RoleAdmin (see common/mod.rs:539)
            let admin_token = get_http_token(
                &ctx.addr,
                common::DEFAULT_USER,
                common::DEFAULT_USER,
                &ctx.client,
            )
            .await
            .unwrap();

            // POST /api/cmd should NOT return 403 for admin
            let resp = ctx
                .test_server
                .post("/api/cmd")
                .with_token(&admin_token)
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({"args": ["test"]}))
                .await
                .unwrap();

            assert_ne!(resp.status(), StatusCode::FORBIDDEN);

            // GET /api/cmd/{job_id} should NOT return 403 for admin
            let fake_job_id = Uuid::new_v4().to_string();
            let resp = ctx
                .test_server
                .get(format!("/api/cmd/{}", fake_job_id))
                .with_token(&admin_token)
                .send()
                .await
                .unwrap();

            assert_ne!(resp.status(), StatusCode::FORBIDDEN);

            // DELETE /api/cmd/{job_id} should NOT return 403 for admin
            let resp = ctx
                .test_server
                .delete(format!("/api/cmd/{}", fake_job_id))
                .with_token(&admin_token)
                .send()
                .await
                .unwrap();

            assert_ne!(resp.status(), StatusCode::FORBIDDEN);
        }
    }

    mod self_service_route_access {
        use crate::common;

        use crate::common::{get_http_token, TestContext, WithToken};

        use super::*;

        /// Verifies that regular users (RoleUser) CAN access self-service routes.
        /// This is the complement to admin_route_access tests — confirms role
        /// enforcement doesn't over-block non-admin users on permitted routes.
        /// Each test uses fresh testcontainers (isolated DB per test), so no
        /// manual cleanup of registered users is needed.
        #[actix_rt::test]
        async fn should_allow_regular_user_on_get_my_profile() {
            let ctx = TestContext::new(None).await;

            // Register a regular user (RoleUser)
            let _ = common::create_http_user(
                &ctx.addr,
                "selfserviceuser",
                "testpass",
                &ctx.client,
            )
            .await;

            // Login as regular user
            let token = get_http_token(
                &ctx.addr,
                "selfserviceuser",
                "testpass",
                &ctx.client,
            )
            .await
            .unwrap();

            // GET /api/user/me should succeed (200) for any authenticated user
            let resp = ctx
                .test_server
                .get("/api/user/me")
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    mod admin_user_route_access {
        use crate::common;

        use crate::common::{get_http_token, TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_return_403_for_non_admin_on_get_users() {
            let ctx = TestContext::new(None).await;

            let _ = common::create_http_user(
                &ctx.addr,
                "nonadminuser1",
                "testpass",
                &ctx.client,
            )
            .await;

            let token = get_http_token(
                &ctx.addr,
                "nonadminuser1",
                "testpass",
                &ctx.client,
            )
            .await
            .unwrap();

            let resp = ctx
                .test_server
                .get("/api/admin/users?page=0&limit=10")
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }

        #[actix_rt::test]
        async fn should_return_403_for_non_admin_on_search_users() {
            let ctx = TestContext::new(None).await;

            let _ = common::create_http_user(
                &ctx.addr,
                "nonadminuser2",
                "testpass",
                &ctx.client,
            )
            .await;

            let token = get_http_token(
                &ctx.addr,
                "nonadminuser2",
                "testpass",
                &ctx.client,
            )
            .await
            .unwrap();

            let resp = ctx
                .test_server
                .get("/api/admin/users/search?q=test&page=0&limit=10")
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }

        #[actix_rt::test]
        async fn should_return_403_for_non_admin_on_get_user() {
            let ctx = TestContext::new(None).await;

            let _ = common::create_http_user(
                &ctx.addr,
                "nonadminuser3",
                "testpass",
                &ctx.client,
            )
            .await;

            let token = get_http_token(
                &ctx.addr,
                "nonadminuser3",
                "testpass",
                &ctx.client,
            )
            .await
            .unwrap();

            let resp = ctx
                .test_server
                .get("/api/admin/users/55")
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }

        #[actix_rt::test]
        async fn should_allow_admin_on_admin_user_routes() {
            let ctx = TestContext::new(None).await;

            let admin_token = get_http_token(
                &ctx.addr,
                common::DEFAULT_USER,
                common::DEFAULT_USER,
                &ctx.client,
            )
            .await
            .unwrap();

            let resp = ctx
                .test_server
                .get("/api/admin/users")
                .with_token(&admin_token)
                .send()
                .await
                .unwrap();

            assert_ne!(resp.status(), StatusCode::FORBIDDEN);
        }
    }
}
