use crate::common;
use actix_demo::get_build_info;

#[cfg(test)]
mod tests {

    use crate::common::TestAppOptions;

    use super::*;
    use actix_web::dev::Service as _;
    use actix_web::http::StatusCode;
    use actix_web::test;

    #[actix_rt::test]
    async fn get_build_info_should_succeed() {
        let connspec = common::pg_conn_string().unwrap();
        let req = test::TestRequest::get()
            .uri("/api/public/build-info")
            .to_request();
        let test_app = common::test_app(&connspec, TestAppOptions::default())
            .await
            .unwrap();
        let resp = test_app.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: build_info::BuildInfo = test::read_body_json(resp).await;
        let _ = tracing::debug!("{:?}", body);
        assert_eq!(body, *get_build_info());
    }
}
