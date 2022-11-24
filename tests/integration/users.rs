use crate::common;
#[cfg(test)]
mod tests {

    use super::*;
    use actix_demo::models::ApiResponse;
    use actix_web::dev::Service as _;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use testcontainers::clients;

    mod get_users_api {

        use std::str::FromStr;

        use actix_demo::models::{roles::RoleEnum, User, UserId, Username};
        use validators::traits::ValidateString;

        use super::*;

        #[actix_rt::test]
        async fn should_return_a_user() {
            let docker = clients::Cli::default();
            let (connspec, _port, _node) = common::start_pg_container(&docker);
            let test_app = common::test_app(&connspec).await.unwrap();
            // let (client, _connection) =
            //     tokio_postgres::connect(&connspec, NoTls).await.unwrap();
            let token = common::get_token(&test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/users?page=0&limit=2")
                .append_header((
                    "Authorization",
                    format! {"Bearer {}", token.to_str().unwrap()},
                ))
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body: ApiResponse<Vec<User>> = test::read_body_json(resp).await;
            let user = body.response().get(0).unwrap();
            assert_eq!(user.id, UserId::from_str("1").unwrap());
            assert_eq!(user.username, Username::parse_str("user1").unwrap());
            assert_eq!(user.role, RoleEnum::RoleUser);
        }

        #[actix_rt::test]
        async fn should_return_error_message_if_user_with_id_does_not_exist() {
            let docker = clients::Cli::default();
            let (connspec, _port, _node) = common::start_pg_container(&docker);
            let test_app = common::test_app(&connspec).await.unwrap();
            let token = common::get_token(&test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/users/55")
                .append_header((
                    "Authorization",
                    format! {"Bearer {}", token.to_str().unwrap()},
                ))
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            let body: ApiResponse<String> = test::read_body_json(resp).await;
            let _ = tracing::debug!("{:?}", body);
            assert_eq!(
                body,
                ApiResponse::failure(
                    "Entity does not exist - No user found with uid: 55"
                        .to_owned()
                )
            );
        }
    }
}
