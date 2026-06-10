use crate::common;
use actix_demo::get_build_info;

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::common::{failing_bin_file, TestAppOptionsBuilder, WithToken};

    use super::*;
    use actix_demo::models::misc::{Job, JobStatus};
    use actix_demo::utils;
    use actix_http::header;
    use actix_http::StatusCode;
    use actix_rt::time::sleep;

    #[actix_rt::test]
    async fn get_build_info_should_succeed() {
        let ctx = common::TestContext::new(None).await;
        let mut resp = ctx
            .test_server
            .get("/api/v1/public/build-info")
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body: build_info::BuildInfo = resp.json().await.unwrap();
        assert_eq!(body, *get_build_info());
    }

    #[actix_rt::test]
    async fn failed_job_test() {
        let res: anyhow::Result<()> = async {
            let file = failing_bin_file();
            let options = TestAppOptionsBuilder::default()
                .bin_file(file)
                .build()
                .unwrap();
            let ctx = common::TestContext::new(Some(options)).await;
            let token = common::get_http_token(
                &ctx.addr,
                common::DEFAULT_USER,
                common::DEFAULT_USER,
                &ctx.client,
            )
            .await
            .unwrap();
            let jwt_key = common::TEST_JWT_KEY.clone();

            let claims = utils::get_claims(&jwt_key, &token)?;
            let user_uuid = claims.custom.user_uuid;
            let current_user_id = {
                let mut conn = ctx.app_data.pool.get().unwrap();
                actix_demo::actions::users::resolve_user_id_by_uuid(
                    &user_uuid, &mut conn,
                )
                .unwrap()
                .unwrap()
            };
            let mut resp = ctx
                .test_server
                .post("/api/v1/cmd")
                .append_header((header::CONTENT_TYPE, "application/json"))
                .with_token(&token)
                .send_body(r#"{"args":[]}"#)
                .await
                .unwrap();
            let job_resp = resp.json::<Job>().await.unwrap();
            assert_eq!(job_resp.started_by, current_user_id);
            assert_eq!(job_resp.status, JobStatus::Pending);

            let job_id = job_resp.job_id.to_string();

            sleep(Duration::from_millis(500)).await;

            let mut resp = ctx
                .test_server
                .get(format!("/api/v1/cmd/{job_id}"))
                .with_token(&token)
                .send()
                .await
                .unwrap();
            let job_resp = resp.json::<Job>().await.unwrap();

            assert_eq!(job_resp.started_by, current_user_id);
            assert_eq!(job_resp.status, JobStatus::Failed);
            Ok(())
        }
        .await;

        tracing::info!("Ended with {res:?}");
        res.unwrap();
    }
}
