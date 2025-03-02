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

            let test_app = common::test_app(
                &pg_connstr,
                &redis_connstr,
                TestAppOptions::default(),
            )
            .await
            .unwrap();
            let token = common::get_default_token(&test_app).await;
            let _ = common::create_user("user1", "test", &test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/users?page=0&limit=2")
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

            let test_app = common::test_app(
                &pg_connstr,
                &redis_connstr,
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
                .uri("/api/users?page=0&limit=10")
                .with_token(&token)
                .to_request();
            let resp = test_app.call(req).await.unwrap();

            // assert size of response is 10
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = test::read_body_json(resp).await;
            assert_eq!(body.len(), 10);

            let req = test::TestRequest::get()
                .uri("/api/users?page=1&limit=10")
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
            let test_app = common::test_app(
                &pg_connstr,
                &redis_connstr,
                TestAppOptions::default(),
            )
            .await
            .unwrap();
            let token = common::get_default_token(&test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/users/55")
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

    mod rate_limiting {
        use crate::common;

        use super::*;
        use actix_demo::models::rate_limit::RateLimitPolicy;
        use actix_http::{header, StatusCode};
        use anyhow::anyhow;
        use awc::Client;
        use std::time::Duration;

        #[actix_rt::test]
        async fn should_rate_limit_api_requests() {
            let res: anyhow::Result<()> = async {
                let (pg_connstr, _pg) = common::test_with_postgres().await?;
                let (redis_connstr, _redis) = common::test_with_redis().await?;
                let options = common::TestAppOptionsBuilder::default()
                .api_rate_limit(RateLimitPolicy {
                    max_requests: 2,
                    window_secs: 2
                })
                .rate_limit_disabled(false)
                .build()
                .unwrap();
            
                let test_server = common::test_http_app(
                    &pg_connstr,
                    &redis_connstr,
                    options
                )
                .await?;

                let addr = test_server.addr().to_string();
                let client = Client::new();

                // Create test user and get token
                let username = "testuser";
                let password = "testpass";
                common::create_http_user(&addr, username, password, &client).await?;
                let token = common::get_http_token(&addr, username, password, &client).await?;

                // Send 2 valid requests
                for _ in 0..2 {
                    let resp = client
                        .get(format!("http://{addr}/api/users?page=0&limit=5"))
                        .append_header((header::CONTENT_TYPE, "application/json"))
                        .with_token(&token)
                        .send()
                        .await
                        .map_err(|err| anyhow!("{err}"))?;

                    assert_eq!(
                        resp.status(),
                        StatusCode::OK,
                        "Expected 200 OK for valid API request"
                    );

                    let headers = resp.headers();
                    println!("Response headers: {:?}", headers);

                    common::assert_rate_limit_headers(headers);
                }

                // Send 3rd request which should be rate limited
                let resp = client
                    .get(format!("http://{addr}/api/users"))
                    .append_header((header::CONTENT_TYPE, "application/json"))
                    .with_token(&token)
                    .send()
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                let headers = resp.headers();
                println!("Response headers: {:?}", headers);

                common::assert_rate_limit_headers(headers);

                assert_eq!(
                    resp.status(),
                    StatusCode::TOO_MANY_REQUESTS,
                    "Expected 429 Too Many Requests after rate limit exceeded"
                );

                // Optional: Test rate limit expiration
                // Sleep for window_secs + 1 seconds to allow rate limit window to expire
                let _ = tokio::time::sleep(Duration::from_secs(3)).await;

                // Try API request after window expiration
                let resp = client
                    .get(format!("http://{addr}/api/users?page=0&limit=5"))
                    .append_header((header::CONTENT_TYPE, "application/json"))
                    .with_token(&token)
                    .send()
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "Expected successful API request after rate limit window expired"
                );

                let headers = resp.headers();
                println!("Response headers: {:?}", headers);

                common::assert_rate_limit_headers(headers);

                Ok(())
            }
            .await;

            tracing::info!("{res:?}");
            res.unwrap();
        }
    }
}
