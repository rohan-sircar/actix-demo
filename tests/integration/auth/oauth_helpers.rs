use actix_demo::config::OAuthConfig;
use awc::Client;
use wiremock::MockServer;

use crate::common::{test_http_app, TestAppOptionsBuilder, TestContext};
use crate::common::{test_with_minio, test_with_postgres, test_with_redis};

pub async fn setup_oauth_app(mock_server: &MockServer) -> TestContext {
    setup_oauth_app_with_servers(mock_server, mock_server).await
}

pub async fn setup_oauth_app_with_servers(
    github_server: &MockServer,
    _google_server: &MockServer,
) -> TestContext {
    let (pg_connstr, _pg) = test_with_postgres().await.unwrap();
    let (redis_connstr, _redis) = test_with_redis().await.unwrap();
    let (minio_connstr, _minio) = test_with_minio().await.unwrap();

    let oauth_config = OAuthConfig {
        enabled: true,
        base_url: format!("http://{}", github_server.address()),
        github_client_id: "test-github-client-id".to_string(),
        github_client_secret: "test-github-client-secret".to_string(),
        github_scopes: vec!["user:email".to_string()],
        google_client_id: "test-google-client-id".to_string(),
        google_client_secret: "test-google-client-secret".to_string(),
        google_scopes: vec!["openid email profile".to_string()],
    };

    let options = TestAppOptionsBuilder::default()
        .smtp_host(Some("localhost:1025".to_string()))
        .oauth_config(Some(oauth_config))
        .build()
        .unwrap();

    let (test_server, app_data) =
        test_http_app(&pg_connstr, &redis_connstr, &minio_connstr, options)
            .await
            .unwrap();

    let addr = test_server.addr().to_string();
    let client = Client::builder().disable_redirects().finish();

    TestContext {
        addr,
        _token: String::new(),
        client,
        _pg,
        _redis,
        _minio,
        test_server,
        app_data,
        _mailpit: None,
        mailpit_client: None,
    }
}
