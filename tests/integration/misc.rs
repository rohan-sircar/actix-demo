use crate::common;
use actix_demo::get_build_info;

#[cfg(test)]
mod tests {

    use super::*;
    use actix_web::dev::Service as _;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use testcontainers::clients;

    #[actix_rt::test]
    async fn get_build_info_should_succeed() {
        let docker = clients::Cli::default();
        let (connspec, _port, _node) = common::start_pg_container(&docker);
        let req = test::TestRequest::get()
            .uri("/api/public/build-info")
            .to_request();
        let test_app = common::test_app(&connspec).await.unwrap();
        let resp = test_app.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: build_info::BuildInfo = test::read_body_json(resp).await;
        let _ = tracing::debug!("{:?}", body);
        assert_eq!(body, *get_build_info());
    }
}
