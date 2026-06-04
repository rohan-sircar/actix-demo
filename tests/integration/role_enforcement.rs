

#[cfg(test)]
mod tests {
    use actix_http::header;
    use actix_web::http::StatusCode;

    mod admin_route_access {
        use crate::common;

        use crate::common::{get_http_token, TestContext, WithToken};

        use super::*;

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

        #[actix_rt::test]
        async fn should_allow_admin_on_admin_routes() {
            use actix_web::http::StatusCode;
            use uuid::Uuid;

            let ctx = TestContext::new(None).await;

            // Admin is the default user, get their token
            let admin_token = get_http_token(
                &ctx.addr,
                common::DEFAULT_USER,
                common::DEFAULT_USER,
                &ctx.client,
            )
            .await
            .unwrap();

            // POST /api/cmd should NOT return 403 for admin
            // (it may fail for other reasons like missing bin, but not 403)
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
}
