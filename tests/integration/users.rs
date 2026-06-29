use crate::common;
#[cfg(test)]
mod tests {

    use super::*;
    use actix_demo::models::misc::ErrorResponse;
    use actix_web::http::StatusCode;
    use std::str::FromStr;

    mod get_users_api {

        use actix_demo::models::{roles::RoleEnum, users::UserWithRoles};

        use crate::common::{get_http_token, TestContext, WithToken};

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

            let token = get_http_token(
                &ctx.addr,
                common::DEFAULT_USER,
                common::DEFAULT_USER,
                &ctx.client,
            )
            .await
            .unwrap();

            let mut resp = ctx
                .test_server
                .get("/api/v1/private/admin/users?page=0&limit=2")
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = resp.json().await.unwrap();
            let user = body.first().unwrap();
            assert_eq!(user.username.as_str(), "admin");
            assert_eq!(user.roles, vec![RoleEnum::RoleAdmin]);
            let user = body.get(1).unwrap();
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

            let token = get_http_token(
                &ctx.addr,
                common::DEFAULT_USER,
                common::DEFAULT_USER,
                &ctx.client,
            )
            .await
            .unwrap();

            // First page with 10 users
            let mut resp = ctx
                .test_server
                .get("/api/v1/private/admin/users?page=0&limit=10")
                .with_token(&token)
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = resp.json().await.unwrap();
            assert_eq!(body.len(), 10);

            // Second page with > 1 user
            let mut resp = ctx
                .test_server
                .get("/api/v1/private/admin/users?page=1&limit=10")
                .with_token(&token)
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

            let token = get_http_token(
                &ctx.addr,
                common::DEFAULT_USER,
                common::DEFAULT_USER,
                &ctx.client,
            )
            .await
            .unwrap();

            let non_existent_uuid =
                actix_demo::models::users::UserUuid::from_str(
                    "00000000-0000-0000-0000-000000000055",
                )
                .unwrap();

            let mut resp = ctx
                .test_server
                .get(format!(
                    "/api/v1/private/admin/users/{}",
                    non_existent_uuid
                ))
                .with_token(&token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            let body: ErrorResponse<String> = resp.json().await.unwrap();
            let _ = tracing::debug!("{:?}", body);
            assert_eq!(
                &body.cause,
                "Entity does not exist - No user found with uuid: 00000000-0000-0000-0000-000000000055"
            );
        }

        mod profile_api {
            use actix_demo::schema::users;
            use actix_web::http::header::CONTENT_TYPE;
            use diesel::prelude::*;

            use super::*;
            use crate::common::register_and_login;

            #[actix_rt::test]
            async fn get_user_profile_returns_empty_when_none_exists() {
                let ctx = TestContext::new(None).await;
                let token =
                    register_and_login(&ctx, "emptyprofile", "test123").await;

                let mut resp = ctx
                    .test_server
                    .get("/api/v1/private/user/profile")
                    .with_token(&token)
                    .send()
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::OK);
                let body: serde_json::Value = resp.json().await.unwrap();
                assert!(body["bio"].is_null());
                assert!(body["display_name"].is_null());
                assert!(body["location"].is_null());
                assert!(body["website_url"].is_null());
                assert!(body["social_github"].is_null());
                assert!(body["social_twitter"].is_null());
            }

            #[actix_rt::test]
            async fn create_profile_success() {
                let ctx = TestContext::new(None).await;
                let token =
                    register_and_login(&ctx, "newprofile", "test123").await;

                let mut resp = ctx
                    .test_server
                    .post("/api/v1/private/user/profile")
                    .with_token(&token)
                    .append_header((CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "bio": "Hello world",
                        "display_name": "Test User",
                        "location": "NYC",
                        "website_url": "https://example.com",
                        "social_github": "githubuser",
                        "social_twitter": "twitteruser"
                    }))
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::CREATED);
                let body: serde_json::Value = resp.json().await.unwrap();
                assert_eq!(body["bio"], "Hello world");
                assert_eq!(body["display_name"], "Test User");
                assert_eq!(body["location"], "NYC");
                assert_eq!(body["website_url"], "https://example.com");
                assert_eq!(body["social_github"], "githubuser");
                assert_eq!(body["social_twitter"], "twitteruser");

                let mut get_resp = ctx
                    .test_server
                    .get("/api/v1/private/user/profile")
                    .with_token(&token)
                    .send()
                    .await
                    .unwrap();
                assert_eq!(get_resp.status(), StatusCode::OK);
                let persisted: serde_json::Value =
                    get_resp.json().await.unwrap();
                assert_eq!(persisted["bio"], "Hello world");
                assert_eq!(persisted["display_name"], "Test User");
            }

            #[actix_rt::test]
            async fn update_profile_partial_only_changes_sent_fields() {
                let ctx = TestContext::new(None).await;
                let token =
                    register_and_login(&ctx, "partialupdate", "test123").await;

                ctx.test_server
                    .post("/api/v1/private/user/profile")
                    .with_token(&token)
                    .append_header((CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "bio": "old bio",
                        "display_name": "Old Name",
                        "location": "Old Location",
                        "website_url": "https://old.com",
                        "social_github": "oldgithub",
                        "social_twitter": "oldtwitter"
                    }))
                    .await
                    .unwrap();

                let mut resp = ctx
                    .test_server
                    .patch("/api/v1/private/user/profile")
                    .with_token(&token)
                    .append_header((CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "bio": "new bio"
                    }))
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::OK);
                let body: serde_json::Value = resp.json().await.unwrap();
                assert_eq!(body["bio"], "new bio");
                assert_eq!(body["display_name"], "Old Name");
                assert_eq!(body["location"], "Old Location");
                assert_eq!(body["website_url"], "https://old.com");
                assert_eq!(body["social_github"], "oldgithub");
                assert_eq!(body["social_twitter"], "oldtwitter");
            }

            #[actix_rt::test]
            async fn update_profile_clears_field_with_null() {
                let ctx = TestContext::new(None).await;
                let token =
                    register_and_login(&ctx, "clearfield", "test123").await;

                ctx.test_server
                    .post("/api/v1/private/user/profile")
                    .with_token(&token)
                    .append_header((CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "bio": "some bio",
                        "display_name": "Some Name",
                        "location": "Some Location",
                        "website_url": "https://some.com",
                        "social_github": "somegithub",
                        "social_twitter": "sometwitter"
                    }))
                    .await
                    .unwrap();

                let mut resp = ctx
                    .test_server
                    .patch("/api/v1/private/user/profile")
                    .with_token(&token)
                    .append_header((CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "bio": null
                    }))
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::OK);
                let body: serde_json::Value = resp.json().await.unwrap();
                assert!(body["bio"].is_null());
                assert_eq!(body["display_name"], "Some Name");
                assert_eq!(body["location"], "Some Location");
            }

            #[actix_rt::test]
            async fn get_public_profile_returns_404_when_missing() {
                let ctx = TestContext::new(None).await;
                let admin_token = common::get_http_token(
                    &ctx.addr,
                    common::DEFAULT_USER,
                    common::DEFAULT_USER,
                    &ctx.client,
                )
                .await
                .unwrap();

                let mut conn = ctx.app_data.pool.get().unwrap();

                let user_uuid: actix_demo::models::users::UserUuid =
                    users::table
                        .filter(users::username.eq("admin"))
                        .select(users::user_uuid)
                        .first(&mut conn)
                        .unwrap();

                let resp = ctx
                    .test_server
                    .get(format!("/api/v1/profiles/{}", user_uuid))
                    .with_token(&admin_token)
                    .send()
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            }

            #[actix_rt::test]
            async fn get_public_profile_returns_data_when_exists() {
                let ctx = TestContext::new(None).await;
                let token =
                    register_and_login(&ctx, "pubprofile", "test123").await;

                ctx.test_server
                    .post("/api/v1/private/user/profile")
                    .with_token(&token)
                    .append_header((CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "bio": "public bio",
                        "display_name": "Public Name",
                        "location": null,
                        "website_url": null,
                        "social_github": null,
                        "social_twitter": null
                    }))
                    .await
                    .unwrap();

                let mut conn = ctx.app_data.pool.get().unwrap();

                let user_uuid: actix_demo::models::users::UserUuid =
                    users::table
                        .filter(users::username.eq("pubprofile"))
                        .select(users::user_uuid)
                        .first(&mut conn)
                        .unwrap();

                let admin_token = common::get_http_token(
                    &ctx.addr,
                    common::DEFAULT_USER,
                    common::DEFAULT_USER,
                    &ctx.client,
                )
                .await
                .unwrap();

                let mut resp = ctx
                    .test_server
                    .get(format!("/api/v1/profiles/{}", user_uuid))
                    .with_token(&admin_token)
                    .send()
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::OK);
                let body: serde_json::Value = resp.json().await.unwrap();
                assert_eq!(body["bio"], "public bio");
                assert_eq!(body["display_name"], "Public Name");
            }

            #[actix_rt::test]
            async fn unauthenticated_get_my_profile_returns_401() {
                let ctx = TestContext::new(None).await;

                let resp = ctx
                    .test_server
                    .get("/api/v1/private/user/profile")
                    .send()
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            }

            #[actix_rt::test]
            async fn unauthenticated_patch_my_profile_returns_401() {
                let ctx = TestContext::new(None).await;

                let resp = ctx
                    .test_server
                    .patch("/api/v1/private/user/profile")
                    .append_header((CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "bio": "test"
                    }))
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            }

            #[actix_rt::test]
            async fn validation_rejects_overlong_bio() {
                let ctx = TestContext::new(None).await;
                let token =
                    register_and_login(&ctx, "longbio", "test123").await;

                let long_bio = "a".repeat(501);

                let resp = ctx
                    .test_server
                    .post("/api/v1/private/user/profile")
                    .with_token(&token)
                    .append_header((CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "bio": long_bio,
                        "display_name": "Test"
                    }))
                    .await
                    .unwrap();

                assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            }
        }
    }
}
