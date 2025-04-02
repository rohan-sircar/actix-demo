use actix_web::{web, HttpRequest, HttpResponse};
use std::convert::From;

use crate::models::misc::{Pagination, SearchQuery};
use crate::models::users::{NewUser, UserId};
use crate::{actions, utils};
use crate::{errors::DomainError, AppData};
use futures::StreamExt;

/// Finds user by UID.
#[tracing::instrument(level = "info", skip(app_data))]
// #[has_any_role("RoleEnum::RoleAdmin", type = "RoleEnum")]
pub async fn get_user(
    app_data: web::Data<AppData>,
    user_id: web::Path<UserId>,
) -> Result<HttpResponse, DomainError> {
    let user_id = user_id.into_inner();
    let _ = tracing::info!("Getting user with id {user_id}");
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::find_user_by_uid(&user_id, &mut conn)
    })
    .await??;
    let _ = tracing::debug!("{:?}", res);
    if let Some(user) = res {
        let _ = tracing::info!("Found user");
        Ok(HttpResponse::Ok().json(user))
    } else {
        let _ = tracing::warn!("Could not find user");
        let err = DomainError::new_entity_does_not_exist_error(format!(
            "No user found with uid: {}",
            user_id
        ));
        Err(err)
    }
}

#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_users(
    app_data: web::Data<AppData>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, DomainError> {
    let _ = tracing::info!("Paginated users request");
    let users = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let p: Pagination = pagination.into_inner();
        actions::users::get_all_users(&p, &mut conn)
    })
    .await??;

    let _ = tracing::info!("Found {} users", users.len());
    let _ = tracing::debug!("{:?}", users);

    Ok(HttpResponse::Ok().json(users))
}

#[tracing::instrument(level = "info", skip(app_data))]
pub async fn search_users(
    app_data: web::Data<AppData>,
    query: web::Query<SearchQuery>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, DomainError> {
    let _ = tracing::info!("Search users request");
    let users = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let p: Pagination = pagination.into_inner();
        actions::users::search_users(query.q.as_str(), &p, &mut conn)
    })
    .await??;

    let _ = tracing::info!("Found {} users", users.len());
    let _ = tracing::debug!("{:?}", users);

    Ok(HttpResponse::Ok().json(users))
}

/// Inserts a new user
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn add_user(
    app_data: web::Data<AppData>,
    form: web::Json<NewUser>,
) -> Result<HttpResponse, DomainError> {
    let user = web::block(move || {
        let pool = &app_data.pool;
        let user_ids_cache = &app_data.user_ids_cache;
        let mut conn = pool.get()?;
        actions::users::insert_new_regular_user(
            form.0,
            app_data.config.hash_cost,
            user_ids_cache,
            &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!("Created user with id={}", user.id);
    let _ = tracing::debug!("{:?}", user);

    Ok(HttpResponse::Created().json(user))
}

/// Upload user avatar
#[tracing::instrument(level = "info", skip_all)]
pub async fn upload_user_avatar(
    app_data: web::Data<AppData>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, DomainError> {
    // Get user ID from header
    let user_id = utils::extract_user_id_from_header(req.headers())?;

    // Get and validate content type
    let content_type =
        utils::extract_header_value(req.headers(), "content-type")?;

    let _ = tracing::debug!("Received content type: {}", content_type);

    // Validate the image stream
    let full_file = utils::validate_image_stream(
        payload,
        &content_type,
        app_data.config.minio.max_avatar_size_bytes as usize,
    )
    .await?;

    let object_key = format!("avatars/{user_id}");
    // Upload to MinIO
    let _ = app_data
        .minio
        .client
        .put_object()
        .bucket(&app_data.config.minio.bucket_name)
        .key(&object_key)
        .body(full_file.freeze().into())
        .content_type(&content_type)
        .send()
        .await
        .map_err(|err| {
            DomainError::new_file_upload_failed(format!(
                "Avatar upload failed: {err:?}"
            ))
        })?;

    Ok(HttpResponse::Ok().json(object_key))
}

/// Get user avatar
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_user_avatar(
    app_data: web::Data<AppData>,
    user_id: web::Path<UserId>,
) -> Result<HttpResponse, DomainError> {
    let user_id = user_id.into_inner();
    let _ = tracing::info!("Getting avatar for user {user_id}");

    // Get the object from MinIO
    let object = app_data
        .minio
        .client
        .get_object()
        .bucket(&app_data.config.minio.bucket_name)
        .key(format!("avatars/{user_id}"))
        .send()
        .await
        .map_err(|err| DomainError::new_internal_error(format!("{err:?}")))?;

    // Get content type from object metadata
    let content_type = object
        .content_type
        .as_deref()
        .unwrap_or("application/octet-stream");

    // Convert ByteStream to AsyncRead and create a streaming response
    let reader = object.body.into_async_read();
    let stream = tokio_util::io::ReaderStream::new(reader)
        .map(|result| result.map(actix_web::web::Bytes::from));

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .streaming(stream))
}
