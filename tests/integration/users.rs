use crate::common;
#[cfg(test)]
mod tests {

    use super::*;
    use crate::common::get_token;
    use actix_demo::models::ApiResponse;
    use actix_web::dev::Service as _;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use std::str;

    mod get_users_api {

        use super::*;

        #[actix_rt::test]
        async fn should_return_a_user() {
            let test_app = common::test_app().await.unwrap();
            let token = get_token(&test_app).await;
            let req = test::TestRequest::get()
                .uri("/api/users?page=0&limit=2")
                .append_header((
                    "Authorization",
                    format! {"Bearer {}", token.to_str().unwrap()},
                ))
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            // let body: ApiResponse<Vec<_>> = test::read_body_json(resp).await;
            // assert!(!body.response().is_empty());
            let bytes = &test::read_body(resp).await;
            let body = str::from_utf8(bytes).unwrap();
            assert!(!body.is_empty());
        }

        #[actix_rt::test]
        async fn should_return_error_message_if_user_with_id_does_not_exist() {
            let test_app = common::test_app().await.unwrap();
            let token = get_token(&test_app).await;
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
