mod session_renewal;
mod sessions_api;

mod tests {
    mod max_concurrent_sessions {
        use crate::common::TestContext;
        use actix_http::StatusCode;

        #[actix_rt::test]
        async fn should_limit_concurrent_sessions() {
            let mut ctx = TestContext::new(None).await;

            // Create 5 sessions successfully
            let _tokens = ctx.create_concurrent_sessions(5).await.unwrap();

            // Try 6th login which should be rejected
            let (status, _) = ctx.attempt_login("Test Device 6").await;

            assert_eq!(
                status,
                StatusCode::TOO_MANY_REQUESTS,
                "Expected 429 Too Many Requests for exceeding max sessions"
            );
        }
    }
}
