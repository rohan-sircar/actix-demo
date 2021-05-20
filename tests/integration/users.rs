use crate::common;
#[cfg(test)]
mod tests {

    use super::*;
    use actix_demo::models::ApiResponse;
    use actix_web::dev::Service as _;
    use actix_web::http::StatusCode;
    use actix_web::test;

    mod get_users_api {
        use super::*;

        #[actix_rt::test]
        async fn should_return_empty_array_if_no_users_exist() {
            let req = test::TestRequest::get()
                .uri("/api/users?page=0&limit=2")
                .to_request();
            let resp =
                common::test_app().await.unwrap().call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body: ApiResponse<Vec<_>> = test::read_body_json(resp).await;
            let _ = tracing::debug!("{:?}", body);
            assert_eq!(body, ApiResponse::successful(vec![1; 0]));
        }

        #[actix_rt::test]
        async fn should_return_error_message_if_user_with_id_does_not_exist() {
            let req = test::TestRequest::get().uri("/api/users/1").to_request();
            let resp =
                common::test_app().await.unwrap().call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            let body: ApiResponse<String> = test::read_body_json(resp).await;
            let _ = tracing::debug!("{:?}", body);
            assert_eq!(
                body,
                ApiResponse::failure(
                    "Entity does not exist - No user found with uid: 1"
                        .to_owned()
                )
            );
        }
    }
}
