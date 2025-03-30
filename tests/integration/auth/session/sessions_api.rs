mod tests {
    mod sessions_api {

        use crate::common;

        #[actix_rt::test]
        async fn should_work() {
            let mut ctx = common::TestContext::new(None).await;
            let tokens = ctx.create_tokens(20).await;

            // Get initial sessions list
            let sessions = ctx.get_sessions(&tokens[0]).await;
            assert_eq!(sessions.len(), 20, "Expected 20 active sessions");

            // Delete last session
            let session_id = sessions.keys().last().unwrap();
            ctx.delete_session(*session_id, &tokens[0]).await;

            // Verify updated sessions list
            let sessions = ctx.get_sessions(&tokens[0]).await;
            assert_eq!(
                sessions.len(),
                19,
                "Expected 19 active sessions after deletion"
            );
        }
    }
}
