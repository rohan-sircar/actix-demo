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
}
