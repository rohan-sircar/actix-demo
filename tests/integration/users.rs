use crate::common;
#[cfg(test)]
mod tests {

    use super::*;
    use actix_demo::models::misc::ErrorResponse;
    use actix_web::dev::Service as _;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use common::WithToken;

    mod get_users_api {

        use actix_demo::models::{roles::RoleEnum, users::UserWithRoles};

        use crate::common::TestAppOptions;

        use super::*;

        #[tokio::test]
        async fn should_return_a_user() {
            let (pg_connstr, _pg) = common::test_with_postgres().await.unwrap();
            let (redis_connstr, _redis) =
                common::test_with_redis().await.unwrap();
            let (minio_connstr, _minio) =
                common::test_with_minio().await.unwrap();

            let test_app = common::test_app(
                &pg_connstr,
                &redis_connstr,
                &minio_connstr,
                TestAppOptions::default(),
            )
            .await
            .unwrap();
            let token = common::get_default_token(&test_app).await;
            let _ = common::create_user("user1", "test", &test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/public/users?page=0&limit=2")
                .with_token(&token)
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = test::read_body_json(resp).await;
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
        #[tokio::test]
        async fn should_return_a_user_with_pagination() {
            let (pg_connstr, _pg) = common::test_with_postgres().await.unwrap();
            let (redis_connstr, _redis) =
                common::test_with_redis().await.unwrap();
            let (minio_connstr, _minio) =
                common::test_with_minio().await.unwrap();

            let test_app = common::test_app(
                &pg_connstr,
                &redis_connstr,
                &minio_connstr,
                TestAppOptions::default(),
            )
            .await
            .unwrap();
            let token = common::get_default_token(&test_app).await;
            // create 10 users
            for i in 0..10 {
                let _ = common::create_user(
                    &format!("user{}", i),
                    "test",
                    &test_app,
                )
                .await;
            }
            let req = test::TestRequest::get()
                .uri("/api/public/users?page=0&limit=10")
                .with_token(&token)
                .to_request();
            let resp = test_app.call(req).await.unwrap();

            // assert size of response is 10
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = test::read_body_json(resp).await;
            assert_eq!(body.len(), 10);

            let req = test::TestRequest::get()
                .uri("/api/public/users?page=1&limit=10")
                .with_token(&token)
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            // assert size of page 1 is 1
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = test::read_body_json(resp).await;
            assert_eq!(body.len(), 1);
        }

        #[actix_rt::test]
        async fn should_return_error_message_if_user_with_id_does_not_exist() {
            let (pg_connstr, _pg) = common::test_with_postgres().await.unwrap();
            let (redis_connstr, _redis) =
                common::test_with_redis().await.unwrap();
            let (minio_connstr, _minio) =
                common::test_with_minio().await.unwrap();
            let test_app = common::test_app(
                &pg_connstr,
                &redis_connstr,
                &minio_connstr,
                TestAppOptions::default(),
            )
            .await
            .unwrap();
            let token = common::get_default_token(&test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/public/users/55")
                .with_token(&token)
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            let body: ErrorResponse<String> = test::read_body_json(resp).await;
            let _ = tracing::debug!("{:?}", body);
            assert_eq!(
                &body.cause,
                "Entity does not exist - No user found with uid: 55"
            );
        }
    }
}
