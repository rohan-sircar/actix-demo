mod tests {
    mod sessions_api {

        use crate::common;

        #[actix_rt::test]
        async fn should_work() {
            let mut ctx = common::TestContext::new(None).await;
            let tokens = ctx.create_tokens(5).await;

            // Get initial sessions list
            let sessions = ctx.get_sessions(&tokens[0]).await;
            assert_eq!(sessions.len(), 5, "Expected 5 active sessions");

            // Delete last session
            let session_id = sessions.keys().last().unwrap();
            ctx.delete_session(*session_id, &tokens[0]).await;

            // Verify updated sessions list
            let sessions = ctx.get_sessions(&tokens[0]).await;
            assert_eq!(
                sessions.len(),
                4,
                "Expected 4 active sessions after deletion"
            );
        }
    }
}
