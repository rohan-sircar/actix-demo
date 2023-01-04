use std::{cell::RefCell, rc::Rc, str::FromStr};

use actix_web::{web, HttpRequest, HttpResponse};
use futures::StreamExt;
use process_stream::{Process, ProcessExt, ProcessItem};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{info_span, Instrument};

use crate::{
    actions,
    errors::DomainError,
    models::{
        misc::{JobStatus, NewJob},
        users::UserId,
        ws::MyProcessItem,
    },
    utils, AppData,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCommandRequest {
    pub args: Vec<String>,
}

#[tracing::instrument(level = "info", skip_all, fields(payload))]
pub async fn run_command(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    payload: web::Json<RunCommandRequest>,
) -> Result<HttpResponse, DomainError> {
    let mut conn = app_data.redis_conn_manager.clone().unwrap();
    let _ = conn.publish("hc", "hc").await?;
    // let job_id =
    //     uuid::Uuid::from_str("319fe476-c767-4788-96cf-dd5a52006231").unwrap();
    let job_id = uuid::Uuid::new_v4();
    let app_data = app_data.clone();
    let bin_path = app_data.config.job_bin_path.clone();
    let redis_prefix = app_data.redis_prefix.as_ref();
    let job_chan_name = redis_prefix(&format!("job.{job_id}"));
    let abort_chan_name = redis_prefix(&format!("job.{job_id}.abort"));
    let payload = payload.into_inner();
    let args = payload.args;
    let user_id = UserId::from_str(
        req.headers().get("x-auth-user").unwrap().to_str().unwrap(),
    )
    .unwrap();

    let pool = app_data.pool.clone();
    let pool2 = pool.clone();
    let job = web::block(move || {
        let conn = pool2.get()?;
        let nj = NewJob {
            job_id,
            started_by: user_id,
            status: JobStatus::Pending,
            status_message: None,
        };
        actions::misc::create_job(&nj, &conn)
    })
    .await??;

    let pool2 = pool.clone();
    let _ = actix_rt::spawn(
        async move {
            let proc = Rc::new(RefCell::new(Process::new(bin_path)));
            {
                let _ = proc.borrow_mut().args(&args);
            }
            let proc2 = proc.clone();

            let aborted = Rc::new(RefCell::new(false));

            let aborted2 = aborted.clone();
            let aborter = actix_rt::spawn(
                async move {
                    // let _ = tokio::time::sleep(Duration::from_millis(5000)).await;
                    let mut ps =
                        utils::get_pubsub(app_data.into_inner()).await?;
                    let _ = ps.subscribe(&abort_chan_name).await?;
                    let mut r_stream = ps.on_message();
                    while let Some(msg) = r_stream.next().await {
                        let msg =
                            &msg.get_payload::<String>().unwrap_or_default();
                        if msg == "done" {
                            let _ = tracing::debug!("Killing");
                            let _ = proc2.borrow().abort();
                            let pool2 = pool.clone();
                            *aborted2.borrow_mut() = true;
                            web::block(move || {
                                let conn = pool2.get()?;
                                actions::misc::update_job_status(job_id,
                                    JobStatus::Aborted,
                                    Some("Job aborted by user".to_owned()),
                                    &conn)
                            })
                            .await??;
                            break;
                        }
                    }
                    Ok::<(), DomainError>(())
                }
                .instrument(info_span!(
                    "job_abort",
                    job_id = job_id.to_string()
                )),
            );
            let publisher = actix_rt::spawn(
                async move {
                    let mut stream = proc
                        .borrow_mut()
                        .spawn_and_stream()
                        .map_err(|err| {
                            DomainError::new_internal_error(format!(
                                "Failed to run process: {err:?}"
                            ))
                        })?
                        .map(|output| match output {
                            ProcessItem::Output(value) => {
                                MyProcessItem::Line { value }
                            }
                            ProcessItem::Error(cause) => {
                                if cause.starts_with("[ERROR]")
                                    || cause.starts_with("E:")
                                {
                                    MyProcessItem::Error { cause }
                                } else {
                                    MyProcessItem::Line { value: cause }
                                }
                            }
                            ProcessItem::Exit(code) => {
                                MyProcessItem::Done { code }
                            }
                        });
                    while let Some(rcm) = stream.next().await {
                        let _ = println!("{:?}", &rcm);
                        let _ = conn
                            .publish(&job_chan_name, utils::jstr(&rcm))
                            .await?;
                        if let MyProcessItem::Done { code } = rcm {
                            let code = code.parse::<i32>().map_err(|err| {
                                DomainError::new_internal_error(
                                    format!("Expected integer return code, got: {code}, err was: {err}")
                                )
                            })?;
                            if code > 0 {
                                Err(DomainError::new_internal_error(
                                    "Failed to run job".to_owned(),
                                ))?;
                            }
                        }
                    }
                    Ok::<(), DomainError>(())
                }
                .instrument(info_span!(
                    "job_publisher",
                    job_id = job_id.to_string()
                )),
            );
            let res = publisher.await?;
            tracing::info!("Job completed");

            aborter.abort();
            let (status, msg) = match res {
                Ok(_) => (JobStatus::Completed, None),
                Err(err) => {
                    let msg = format!("Error running job: {err:?}");
                    tracing::error!(msg);
                    (JobStatus::Failed, Some(msg))
                }
            };
            if !*aborted.borrow() {
                let conn = pool2.get()?;
                web::block(move || {
                    actions::misc::update_job_status(job_id, status, msg, &conn)
                })
                .await??;
            }
            Ok::<(), DomainError>(())
        }
        .instrument(info_span!("job", job_id = job_id.to_string())),
    );
    Ok(HttpResponse::Ok().json(job))
}

#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_job(
    app_data: web::Data<AppData>,
    job_id: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    let pool = app_data.pool.clone();
    let job_id =
        uuid::Uuid::parse_str(&job_id.into_inner()).map_err(|err| {
            DomainError::new_bad_input_error(format!("Expected UUID: {err}"))
        })?;
    let job = web::block(move || {
        let conn = &pool.get()?;
        actions::misc::get_job_by_uuid(job_id, conn)
    })
    .await??;
    match job {
        Some(job) => {
            tracing::info!("Found job with id: {}", job.job_id);
            tracing::debug!("Found job: {job:?}");
            Ok(HttpResponse::Ok().json(job))
        }
        None => Err(DomainError::new_entity_does_not_exist_error(format!(
            "No jobs with uuid: {job_id}"
        ))),
    }
}

#[tracing::instrument(level = "info", skip(app_data))]
pub async fn abort_command(
    app_data: web::Data<AppData>,
    job_id: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    let mut conn = app_data.redis_conn_manager.clone().unwrap();
    let job_id = job_id.into_inner();
    // let job_id =
    //     uuid::Uuid::from_str("319fe476-c767-4788-96cf-dd5a52006231").unwrap();
    let abort_chan_name =
        (app_data.redis_prefix)(&format!("job.{job_id}.abort"));
    let _ = conn.publish(abort_chan_name, "done").await?;
    Ok(HttpResponse::Ok().finish())
}
