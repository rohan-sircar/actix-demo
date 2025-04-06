mod tests {
    mod sessions_api {

        use actix_demo::models::session::SessionConfigBuilder;

        use crate::common::{self, TestAppOptionsBuilder};

        #[actix_rt::test]
        async fn should_work() {
            let sessions_count = 50;
            let ctx = common::TestContext::new(Some(
                TestAppOptionsBuilder::default()
                    .session_config(
                        SessionConfigBuilder::default()
                            .max_concurrent_sessions(sessions_count + 1)
                            .build()
                            .unwrap(),
                    )
                    .build()
                    .unwrap(),
            ))
            .await;
            let tokens = ctx.create_tokens(sessions_count - 1).await;

            // Get initial sessions list
            let sessions = ctx.get_sessions(&tokens[0]).await;
            assert_eq!(
                sessions.len(),
                sessions_count,
                "Expected {sessions_count} active sessions"
            );

            // Delete last session
            let session_id = sessions.keys().last().unwrap();
            ctx.delete_session(*session_id, &tokens[0]).await;

            // Verify updated sessions list
            let sessions = ctx.get_sessions(&tokens[0]).await;
            assert_eq!(
                sessions.len(),
                sessions_count - 1,
                "Expected {} active sessions after deletion",
                sessions_count - 1
            );
        }
    }
}
