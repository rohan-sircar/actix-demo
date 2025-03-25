mod session_renewal;
mod sessions_api;

mod tests {
    mod max_concurrent_sessions {
        use crate::common::{self, TestAppOptionsBuilder};

        use actix_http::{header, StatusCode};
        use anyhow::anyhow;
        use awc::Client;

        #[actix_rt::test]
        async fn should_limit_concurrent_sessions() {
            let res: anyhow::Result<()> = async {
                // Set up test infrastructure
                let (pg_connstr, _pg) = common::test_with_postgres().await?;
                let (redis_connstr, _redis) = common::test_with_redis().await?;

                // Create test app instance
                let test_server = common::test_http_app(
                    &pg_connstr,
                    &redis_connstr,
                    TestAppOptionsBuilder::default().build().unwrap(),
                )
                .await?;

                let addr = test_server.addr().to_string();
                let client = Client::new();

                // Create test user
                let username = "session.test.user";
                let password = "test_password";

                common::create_http_user(&addr, username, password, &client)
                    .await?;

                // Perform 5 successful logins
                for i in 0..5 {
                    let resp = client
                        .post(format!("http://{addr}/api/login"))
                        .append_header((
                            header::CONTENT_TYPE,
                            "application/json",
                        ))
                        .send_json(&serde_json::json!({
                            "username": username,
                            "password": password,
                            // "device_id": format!("device_{i}"),
                            "device_name": format!("Test Device {i}")
                        }))
                        .await
                        .map_err(|err| anyhow!("{err}"))?;

                    assert_eq!(
                        resp.status(),
                        StatusCode::OK,
                        "Expected successful login for attempt {}",
                        i + 1
                    );
                }

                // Try 6th login which should be rejected
                let resp = client
                    .post(format!("http://{addr}/api/login"))
                    .append_header((header::CONTENT_TYPE, "application/json"))
                    .send_json(&serde_json::json!({
                        "username": username,
                        "password": password,
                        // "device_id": "device_6",
                        "device_name": "Test Device 6"
                    }))
                    .await
                    .map_err(|err| anyhow!("{err}"))?;

                assert_eq!(
                    resp.status(),
                    StatusCode::TOO_MANY_REQUESTS,
                    "Expected 429 Too Many Requests for exceeding max sessions"
                );

                Ok(())
            }
            .await;

            tracing::info!("{res:?}");
            res.unwrap();
        }
    }
}
