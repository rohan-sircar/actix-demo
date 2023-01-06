use crate::common;
use actix_demo::get_build_info;

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::common::{
        failing_bin_file, TestAppOptions, TestAppOptionsBuilder, WithToken,
    };

    use super::*;
    use actix_demo::models::misc::{Job, JobStatus};
    use actix_http::header;
    use actix_rt::time::sleep;
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

    #[actix_rt::test]
    async fn failed_job_test() {
        let res = async {
            let connspec = common::pg_conn_string()?;
            let file = failing_bin_file();
            let options = TestAppOptionsBuilder::default()
                .bin_file(file)
                .build()
                .unwrap();
            let test_app = common::test_app(&connspec, options).await.unwrap();
            let token = common::get_default_token(&test_app).await;
            let req = test::TestRequest::post()
                .append_header((header::CONTENT_TYPE, "application/json"))
                .uri("/api/cmd")
                .with_token(token.clone())
                .set_payload(r#"{"args":[]}"#.as_bytes())
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let job_resp: Job = test::read_body_json(resp).await;

            let job_id = job_resp.job_id.to_string();
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Pending);

            sleep(Duration::from_millis(500)).await;

            let req = test::TestRequest::get()
                .uri(&format!("/api/cmd/{job_id}"))
                .with_token(token.clone())
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let job_resp: Job = test::read_body_json(resp).await;

            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Failed);
            Ok::<(), anyhow::Error>(())
        }
        .await;

        tracing::info!("{res:?}");
        res.unwrap();
    }
}
