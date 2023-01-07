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
            let connspec = common::pg_conn_string().unwrap();
            let test_app =
                common::test_app(&connspec, TestAppOptions::default())
                    .await
                    .unwrap();
            // let (client, _connection) =
            //     tokio_postgres::connect(&connspec, NoTls).await.unwrap();
            let token = common::get_default_token(&test_app).await;
            let _ = common::create_user("user1", "test", &test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/users?page=0&limit=2")
                .with_token(token)
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Vec<UserWithRoles> = test::read_body_json(resp).await;
            let user = body.get(0).unwrap();
            assert_eq!(user.id.as_uint(), 1);
            assert_eq!(user.username.as_str(), "admin");
            assert_eq!(user.roles, vec![RoleEnum::RoleAdmin]);
            let user = body.get(1).unwrap();
            assert_eq!(user.id.as_uint(), 2);
            assert_eq!(user.username.as_str(), "user1");
            assert_eq!(user.roles, vec![RoleEnum::RoleUser]);
        }

        #[actix_rt::test]
        async fn should_return_error_message_if_user_with_id_does_not_exist() {
            let connspec = common::pg_conn_string().unwrap();
            let test_app =
                common::test_app(&connspec, TestAppOptions::default())
                    .await
                    .unwrap();
            let token = common::get_default_token(&test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/users/55")
                .with_token(token)
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
