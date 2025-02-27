use crate::common;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_http::{header, StatusCode};
    use anyhow::anyhow;
    use awc::Client;

    use common::TestAppOptions;
    use std::time::Duration;

    mod login_rate_limiting {
        use super::*;

        #[actix_rt::test]
        async fn should_rate_limit_failed_login_attempts() {
            let res: anyhow::Result<()> = async {
                // Set up test infrastructure
                let (pg_connstr, _pg) = common::test_with_postgres().await?;
                let (redis_connstr, _redis) = common::test_with_redis().await?;

                // Create test app instance
                let test_server = common::test_http_app(
                    &pg_connstr,
                    &redis_connstr,
                    TestAppOptions::default(),
                )
                .await?;

                let addr = test_server.addr().to_string();
                let client = Client::new();

                // Create test user
                let username = "testuser1";
                let correct_password = "correct_password";

                common::create_http_user(
                    &addr,
                    username,
                    correct_password,
                    &client,
                )
                .await?;

                // Send 5 failed login attempts
                let wrong_password = "wrong_password";
                for _ in 0..5 {
                    let resp = client
                        .post(format!("http://{addr}/api/login"))
                        .append_header((
                            header::CONTENT_TYPE,
                            "application/json",
                        ))
                        .send_json(&serde_json::json!({
                            "username": username,
                            "password": wrong_password
                        }))
                        .await
                        .map_err(|err| anyhow!("{err}"))?;

                    assert_eq!(
                        resp.status(),
                        StatusCode::UNAUTHORIZED,
                        "Expected 401 Unauthorized for failed login attempt"
                    );
                }

                // Send 6th login attempt which should be rate limited
                let resp = client
                    .post(format!("http://{addr}/api/login"))
                    .append_header((header::CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "username": username,
                        "password": wrong_password
                    }))
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                assert_eq!(
                    resp.status(),
                    StatusCode::TOO_MANY_REQUESTS,
                    "Expected 429 Too Many Requests after rate limit exceeded"
                );

                // Optional: Test rate limit expiration
                // Sleep for 61 seconds to allow rate limit window to expire
                tokio::time::sleep(Duration::from_secs(3)).await;

                // Try login with correct password after window expiration
                let resp = client
                    .post(format!("http://{addr}/api/login"))
                    .append_header((header::CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "username": username,
                        "password": correct_password
                    }))
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "Expected successful login after rate limit window expired"
                );

                Ok(())
            }
            .await;

            tracing::info!("{res:?}");
            res.unwrap();
        }
    }
}
