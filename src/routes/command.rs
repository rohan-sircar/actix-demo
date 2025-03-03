use std::{cell::RefCell, rc::Rc, str::FromStr};

use actix_web::{web, HttpRequest, HttpResponse};
use futures::StreamExt;
use process_stream::{Process, ProcessExt, ProcessItem};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{info_span, Instrument};
use uuid::Uuid;

use crate::{
    actions,
    errors::DomainError,
    models::{
        misc::{Job, JobStatus, NewJob},
        users::UserId,
        ws::MyProcessItem,
    },
    types::Task,
    utils, AppData,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCommandRequest {
    pub args: Vec<String>,
}

/// Executes a long-running command as a background job
///
/// # Arguments
/// * `req` - HTTP request containing authentication headers
/// * `app_data` - Shared application state
/// * `payload` - JSON payload containing command arguments
///
/// # Returns
/// Returns HTTP 200 with job details if successful, or an error response
///
/// # Process
/// 1. Creates a new job record in database
/// 2. Spawns process with provided arguments
/// 3. Publishes process output to Redis channel
/// 4. Handles job abort requests
/// 5. Updates job status on completion
#[tracing::instrument(level = "info", skip_all, fields(payload))]
pub async fn handle_run_command(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    payload: web::Json<RunCommandRequest>,
) -> Result<HttpResponse, DomainError> {
    tracing::info!("Starting new command execution job");
    let mut conn = app_data.get_redis_conn()?;
    // Health check publish to verify Redis connection
    let () = conn.publish("hc", "hc").await?;

    // Generate unique job ID
    let job_id = uuid::Uuid::new_v4();
    tracing::debug!("Generated new job ID: {}", job_id);
    let app_data = app_data.clone();
    let bin_path = app_data.config.job_bin_path.clone();
    let redis_prefix = app_data.redis_prefix.as_ref();
    let job_chan_name = redis_prefix(&format!("job.{job_id}"));
    let abort_chan_name = redis_prefix(&format!("job.{job_id}.abort"));
    let payload = payload.into_inner();
    let args = payload.args;
    // Extract and validate user ID from auth header
    let user_id = req
        .headers()
        .get("x-auth-user")
        .ok_or_else(|| {
            DomainError::new_auth_error("Missing x-auth-user header".to_owned())
        })
        .and_then(|hv| {
            hv.to_str().map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "x-auth-user header is not a valid UTF-8 string: {err}"
                ))
            })
        })
        .and_then(|str| {
            UserId::from_str(str).map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "Invalid UserId format in x-auth-user header: {err}"
                ))
            })
        })?;
    tracing::debug!("Authenticated user ID: {}", user_id);

    // Create new job record in database
    let pool = app_data.pool.clone();
    let pool2 = pool.clone();
    tracing::debug!("Creating new job record in database");
    let job = web::block(move || {
        let mut conn = pool2.get()?;
        let nj = NewJob {
            job_id,
            started_by: user_id,
            status: JobStatus::Pending,
            status_message: None,
        };
        actions::misc::create_job(&nj, &mut conn)
    })
    .await??;
    tracing::info!("Successfully created job with ID: {}", job.job_id);

    let pool2 = pool.clone();
    let _task: Task<()> = actix_rt::spawn(
        async move {
            // Create and configure process
            let proc = Rc::new(RefCell::new(Process::new(bin_path)));
            {
                tracing::debug!("Setting process arguments: {:?}", args);
                let _ = proc.borrow_mut().args(&args);
            }
            let proc2 = proc.clone();

            // Track abort state
            let aborted = Rc::new(RefCell::new(false));
            tracing::debug!("Initialized abort state tracking");

            // Spawn abort handler task
            let aborted2 = aborted.clone();
            let aborter: Task<()> = actix_rt::spawn(
                async move {
                    // Initialize pubsub connection
                    let mut ps = utils::get_pubsub(app_data.into_inner()).await.map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to initialize pubsub connection: {err}"
                        ))
                    })?;

                    // Subscribe to abort channel
                    let _ = ps.subscribe(&abort_chan_name).await.map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to subscribe to abort channel: {err}"
                        ))
                    })?;

                    // Process incoming messages
                    let mut r_stream = ps.on_message();
                    while let Some(msg) = r_stream.next().await {
                        // Safely handle message payload
                        let msg = msg.get_payload::<String>().unwrap_or_default();

                        if msg == "done" {
                            let _ = tracing::info!("Received abort signal for job {}", job_id);                           
                            // Abort the process
                            let _ = proc2.borrow().abort();
                            // Update abort state
                            *aborted2.borrow_mut() = true;
                            // Update job status in database
                            let pool2 = pool.clone();
                            web::block(move || {
                                let mut conn = pool2.get()?;
                                actions::misc::update_job_status(
                                    job_id,
                                    JobStatus::Aborted,
                                    Some("Job aborted by user".to_owned()),
                                    &mut conn,
                                )
                            })
                            .await
                            .map_err(|err| {
                                DomainError::new_internal_error(format!(
                                    "Failed to update job status: {err}"
                                ))
                            })??;
                            break;
                                                }
                    }
                    Ok(())
                }
                .instrument(info_span!("job_aborter", job_id = job_id.to_string())),
            );
            // Spawn publisher task to handle process output
            let publisher: Task<()> = actix_rt::spawn(
                async move {
                    tracing::debug!("Starting process with arguments");
                    let mut stream = proc
                        .borrow_mut()
                        .spawn_and_stream()
                        .map_err(|err| {
                            tracing::error!("Failed to start process: {:?}", err);
                            DomainError::new_internal_error(format!(
                                "Failed to run process: {err:?}"
                            ))
                        })?
                        .map(|output| match output {
                            ProcessItem::Output(value) => {
                                tracing::trace!("Process output: {}", value);
                                MyProcessItem::Line { value }
                            },
                            ProcessItem::Error(cause) => {
                                if cause.starts_with("[ERROR]") || cause.starts_with("E:") {
                                    tracing::warn!("Process error: {}", cause);
                                    MyProcessItem::Error { cause }
                                } else {
                                    tracing::trace!("Process output: {}", cause);
                                    MyProcessItem::Line { value: cause }
                                }
                            }
                            ProcessItem::Exit(code) => {
                                tracing::info!("Process exited with code: {}", code);
                                MyProcessItem::Done { code }
                            },
                        });

                    // Publish process output to Redis channel
                    while let Some(rcm) = stream.next().await {
                        tracing::trace!("Publishing process output: {:?}", &rcm);
                        let () = conn.publish(&job_chan_name, utils::jstr(&rcm)).await?;
                        // Handle process completion
                        if let MyProcessItem::Done { code } = rcm {
                            let code = code.parse::<i32>().map_err(|err| {
                                tracing::error!("Invalid exit code format: {}", err);
                                DomainError::new_internal_error(format!(
                                    "Expected integer return code, got: {code}, err was: {err}"
                                ))
                            })?;
                            if code > 0 {
                                tracing::error!("Process failed with exit code: {}", code);
                                Err(DomainError::new_internal_error(
                                    "Failed to run job".to_owned(),
                                ))?;
                            }
                        }
                    }
                    tracing::info!("Process output publishing completed");
                    Ok(())
                }
                .instrument(info_span!("job_publisher", job_id = job_id.to_string())),
            );
            // Wait for publisher task to complete
            let res = publisher.await?;
            tracing::info!("Job {} completed", job_id);

            // Clean up abort handler
            aborter.abort();
            tracing::debug!("Abort handler terminated");

            // Update job status in database if not already aborted
            if !*aborted.borrow() {
                // Determine final job status
            let (status, msg) = match res {
                Ok(_) => {
                    tracing::info!("Job {} completed successfully", job_id);
                    (JobStatus::Completed, None)
                },
                Err(err) => {
                    let msg = format!("Error running job: {err:?}");
                    tracing::error!("Job {} failed: {}", job_id, msg);
                    (JobStatus::Failed, Some(msg))
                }
            };
                tracing::debug!("Updating job {} status to {:?}", job_id, status);
                let mut conn = pool2.get()?;
                web::block(move || {
                    actions::misc::update_job_status(job_id, status, msg, &mut conn)
                })
                .await??;
            }
            tracing::info!("Job {} processing complete", job_id);
            Ok(())
        }
        .instrument(info_span!("job", job_id = job_id.to_string())),
    );
    Ok(HttpResponse::Ok().json(job))
}

/// Retrieves a job from the database by its UUID.
///
/// # Arguments
///
/// * `app_data` - Shared application data, including database pool.
/// * `job_id` - Path parameter representing the UUID of the job.
///
/// # Returns
///
/// * `Result<HttpResponse, DomainError>` - HTTP response with job data if found, otherwise an error.
///
/// # Errors
///
/// * `DomainError` - If the provided job ID is not a valid UUID, or if the job does not exist.
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn handle_get_job(
    app_data: web::Data<AppData>,
    job_id: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    // Parse the job ID from the path parameter as a UUID.
    let job_id = Uuid::parse_str(&job_id.into_inner()).map_err(|err| {
        DomainError::new_bad_input_error(format!("Expected UUID: {err}"))
    })?;

    let job = fetch_job_by_uuid(job_id, app_data.as_ref()).await?;

    Ok(HttpResponse::Ok().json(job))
}

async fn fetch_job_by_uuid(
    job_id: Uuid,
    app_data: &AppData,
) -> Result<Job, DomainError> {
    let pool = app_data.pool.clone();

    web::block(move || {
        let mut conn = pool.get()?;
        actions::misc::get_job_by_uuid(job_id, &mut conn)
    })
    .await??
    .ok_or_else(|| {
        DomainError::new_entity_does_not_exist_error(format!(
            "No jobs with uuid: {job_id}"
        ))
    })
}

// You can then call `fetch_job_by_uuid` from your original function

/// Aborts a command by sending a message to the Redis channel associated with the job.
///
/// # Arguments
///
/// * `app_data` - Shared application data, including Redis connection.
/// * `job_id` - Path parameter representing the UUID of the job to abort.
///
/// # Returns
///
/// * `Result<HttpResponse, DomainError>` - HTTP response indicating success or failure.
///
/// # Errors
///
/// * `DomainError` - If there is an error publishing to the Redis channel.
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn handle_abort_job(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    job_id: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    // Extract and validate user ID from auth header
    let user_id = req
        .headers()
        .get("x-auth-user")
        .ok_or_else(|| {
            DomainError::new_auth_error("Missing x-auth-user header".to_owned())
        })
        .and_then(|hv| {
            hv.to_str().map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "x-auth-user header is not a valid UTF-8 string: {err}"
                ))
            })
        })
        .and_then(|str| {
            UserId::from_str(str).map_err(|err| {
                DomainError::new_bad_input_error(format!(
                    "Invalid UserId format in x-auth-user header: {err}"
                ))
            })
        })?;

    // Get a Redis connection from the app_data.
    let mut conn = app_data.get_redis_conn()?;

    // Parse the job ID from the path parameter as a UUID.
    let job_id = Uuid::parse_str(&job_id.into_inner()).map_err(|err| {
        DomainError::new_bad_input_error(format!("Expected UUID: {err}"))
    })?;

    let job = fetch_job_by_uuid(job_id, app_data.as_ref()).await?;

    if user_id != job.started_by {
        return Err(DomainError::new_auth_error(
            "Forbidden: Tried to abort job of a different user".to_owned(),
        ));
    };

    // Construct the Redis channel name for aborting the job.
    let abort_chan_name =
        (app_data.redis_prefix)(&format!("job.{job_id}.abort"));

    // Publish a message to the Redis channel to abort the job.
    let () = conn.publish(abort_chan_name, "done").await?;

    let _ = tracing::info!("Abort command sent for job with id: {}", job_id);

    Ok(HttpResponse::Ok().finish())
}
