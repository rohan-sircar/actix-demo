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
    use actix_demo::utils;
    use actix_http::header;
    use actix_rt::time::sleep;
    use actix_web::dev::Service as _;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use jwt_simple::prelude::HS256Key;

    #[actix_rt::test]
    async fn get_build_info_should_succeed() {
        let (pg_connstr, _pg) = common::test_with_postgres().await.unwrap();
        let (redis_connstr, _redis) = common::test_with_redis().await.unwrap();
        let (minio_connstr, _minio) = common::test_with_minio().await.unwrap();
        let req = test::TestRequest::get()
            .uri("/api/public/build-info")
            .to_request();
        let test_app = common::test_app(
            &pg_connstr,
            &redis_connstr,
            &minio_connstr,
            TestAppOptions::default(),
        )
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
        let res: anyhow::Result<()> = async {
            let (pg_connstr, _pg) = common::test_with_postgres().await?;
            let (redis_connstr, _redis) = common::test_with_redis().await?;
            let (minio_connstr, _minio) = common::test_with_minio().await?;
            let file = failing_bin_file();
            let options = TestAppOptionsBuilder::default()
                .bin_file(file)
                .build()
                .unwrap();
            let test_app = common::test_app(
                &pg_connstr,
                &redis_connstr,
                &minio_connstr,
                options,
            )
            .await
            .unwrap();
            let token = common::get_default_token(&test_app).await;
            let jwt_key = HS256Key::from_bytes("test".as_bytes());

            let claims = utils::get_claims(&jwt_key, &token)?;
            let user_id = claims.custom.user_id;
            let req = test::TestRequest::post()
                .append_header((header::CONTENT_TYPE, "application/json"))
                .uri("/api/cmd")
                .with_token(&token)
                .set_payload(r#"{"args":[]}"#.as_bytes())
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let job_resp: Job = test::read_body_json(resp).await;

            let job_id = job_resp.job_id.to_string();
            assert_eq!(job_resp.started_by, user_id);
            assert_eq!(job_resp.status, JobStatus::Pending);

            sleep(Duration::from_millis(500)).await;

            let req = test::TestRequest::get()
                .uri(&format!("/api/cmd/{job_id}"))
                .with_token(&token)
                .to_request();
            let resp = test_app.call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let job_resp: Job = test::read_body_json(resp).await;

            assert_eq!(job_resp.started_by, user_id);
            assert_eq!(job_resp.status, JobStatus::Failed);
            Ok(())
        }
        .await;

        tracing::info!("Ended with {res:?}");
        res.unwrap();
    }
}
