mod common;

#[cfg(test)]
mod tests {

    use super::*;
    extern crate actix_demo;
    use actix_demo::models::ErrorModel;
    use actix_web::dev::Service as _;
    use actix_web::http::StatusCode;
    use actix_web::test;

    #[actix_rt::test]
    async fn get_users_api_should_succeed() {
        let req = test::TestRequest::get().uri("/api/get/users").to_request();
        let resp = common::test_app().await.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        let body: ErrorModel = test::read_body_json(resp).await;
        assert_eq!(
            body,
            ErrorModel {
                success: false,
                reason: "Entity does not exist - No users available".to_owned()
            }
        );
        log::debug!("{:?}", body);
    }

    #[actix_rt::test]
    async fn get_user_api_should_succeed() {
        let req = test::TestRequest::get()
            .uri("/api/get/users/1")
            .to_request();
        let resp = common::test_app().await.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        let body: ErrorModel = test::read_body_json(resp).await;
        assert_eq!(
            body,
            ErrorModel {
                success: false,
                reason: "Entity does not exist - No user found with uid: 1"
                    .to_owned()
            }
        );
        log::debug!("{:?}", body);
    }
}
