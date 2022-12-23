use std::{cell::RefCell, rc::Rc, time::Duration};

use actix_web::{web, HttpResponse};
use futures::StreamExt;
use process_stream::{Process, ProcessExt, ProcessItem};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{info_span, Instrument};

use crate::{errors::DomainError, models::ws::MyProcessItem, utils, AppData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCommandRequest {
    pub args: Vec<String>,
}

#[tracing::instrument(level = "info", skip(app_data))]
pub async fn run_command(
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

    let _ = actix_rt::spawn(
        async move {
            let _ = tokio::time::sleep(Duration::from_millis(1000)).await;
            let proc = Rc::new(RefCell::new(Process::new(bin_path)));
            {
                let _ = proc.borrow_mut().args(&args);
            }
            let proc2 = proc.clone();

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
                            let _ = proc2.borrow_mut().abort();
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
            let _pub_task = actix_rt::spawn(async move {
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
                        ProcessItem::Exit(code) => MyProcessItem::Done { code },
                    });
                while let Some(rcm) = stream.next().await {
                    // let _ = println!("{:?}", &rcm);
                    let _ =
                        conn.publish(&job_chan_name, utils::jstr(&rcm)).await?;
                    // let _ = if let RunCommandMessage::Error { cause: _ } = &rcm {
                    //     tx.send("done").await.unwrap()
                    // };
                }
                aborter.abort();
                Ok::<(), DomainError>(())
            });
            let res = _pub_task.await?;

            if let Err(err) = res {
                tracing::error!("Error running job: {err:?}");
            }
            Ok::<(), DomainError>(())
        }
        .instrument(info_span!("job", job_id = job_id.to_string())),
    );
    Ok(HttpResponse::Ok().body(job_id.to_string()))
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
