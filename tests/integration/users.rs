use crate::common;
#[cfg(test)]
mod tests {

    use super::*;
    use actix_demo::models::misc::ErrorResponse;
    use actix_web::http::StatusCode;

    mod get_users_api {

        use actix_demo::models::{roles::RoleEnum, users::UserWithRoles};

        use crate::common::TestContext;

        use super::*;

        #[actix_rt::test]
        async fn should_return_a_user() {
            let ctx = TestContext::new(None).await;
            let _ = common::create_http_user(
                &ctx.addr,
                "user1",
                "test",
                &ctx.client,
            )
            .await;

            let mut resp = ctx
                .test_server
                .get("/api/public/users?page=0&limit=2")
                // .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = resp.json().await.unwrap();
            let user = body.first().unwrap();
            assert_eq!(user.id.as_uint(), 1);
            assert_eq!(user.username.as_str(), "admin");
            assert_eq!(user.roles, vec![RoleEnum::RoleAdmin]);
            let user = body.get(1).unwrap();
            assert_eq!(user.id.as_uint(), 2);
            assert_eq!(user.username.as_str(), "user1");
            assert_eq!(user.roles, vec![RoleEnum::RoleUser]);
        }

        // add test for pagination
        #[actix_rt::test]
        async fn should_return_a_user_with_pagination() {
            let ctx = TestContext::new(None).await;

            // create 10 users
            for i in 0..10 {
                let _ = common::create_http_user(
                    &ctx.addr,
                    &format!("user{}", i),
                    "test",
                    &ctx.client,
                )
                .await;
            }

            // First page with 10 users
            let mut resp = ctx
                .test_server
                .get("/api/public/users?page=0&limit=10")
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = resp.json().await.unwrap();
            assert_eq!(body.len(), 10);

            // Second page with > 1 user
            let mut resp = ctx
                .test_server
                .get("/api/public/users?page=1&limit=10")
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = resp.json().await.unwrap();
            assert_eq!(body.len(), 1);
        }

        #[actix_rt::test]
        async fn should_return_error_message_if_user_with_id_does_not_exist() {
            let ctx = TestContext::new(None).await;

            let mut resp = ctx
                .test_server
                .get("/api/public/users/55")
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            let body: ErrorResponse<String> = resp.json().await.unwrap();
            let _ = tracing::debug!("{:?}", body);
            assert_eq!(
                &body.cause,
                "Entity does not exist - No user found with uid: 55"
            );
        }
    }

    mod avatar_api {
        use super::*;
        use crate::common::{TestContext, WithToken};

        // Valid 1x1 pixel transparent PNG image
        static PNG_IMAGE: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
            0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT
            0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, // IEND
            0x42, 0x60, 0x82,
        ];

        #[actix_rt::test]
        async fn should_delete_user_avatar() {
            let ctx = TestContext::new(None).await;

            // First, upload an avatar using admin token
            let mut upload_resp = ctx
                .test_server
                .put("/api/avatars")
                .with_token(&ctx._token)
                .insert_header(("content-type", "image/png"))
                .send_body(PNG_IMAGE)
                .await
                .unwrap();
            assert_eq!(upload_resp.status(), StatusCode::OK);
            let uploaded_key: String = upload_resp.json().await.unwrap();
            assert_eq!(uploaded_key, "avatars/1");

            // Delete the avatar using admin token
            let mut delete_resp = ctx
                .test_server
                .delete("/api/avatars")
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();
            assert_eq!(delete_resp.status(), StatusCode::OK);
            let deleted_key: String = delete_resp.json().await.unwrap();
            assert_eq!(deleted_key, "avatars/1");

            // Verify avatar is gone by trying to GET it
            let get_resp = ctx
                .test_server
                .get("/api/public/users/1/avatar")
                .send()
                .await
                .unwrap();
            assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
        }

        #[actix_rt::test]
        async fn should_return_unauthorized_without_token() {
            let ctx = TestContext::new(None).await;

            let mut resp = ctx
                .test_server
                .delete("/api/avatars")
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            // 401 responses don't have JSON body, just verify the status
            let body = resp.body().await.unwrap();
            assert!(!body.is_empty());
        }

        #[actix_rt::test]
        async fn should_return_ok_when_avatar_does_not_exist() {
            let ctx = TestContext::new(None).await;

            // Try to delete a non-existent avatar using admin token
            let resp = ctx
                .test_server
                .delete("/api/avatars")
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            // MinIO returns success even if the object doesn't exist
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}
