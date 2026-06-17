use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use awc::cookie::{Cookie, SameSite};
use std::str::FromStr;
use time::OffsetDateTime;
use utoipa::ToSchema;

use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use crate::models::misc::Pagination;
use crate::models::roles::RoleEnum;
use crate::models::users::{
    CreateProfile, NewUser, PublicProfile, UpdateProfile, UpdateUserProfile,
    UserUuid,
};
use crate::services::email::tokens;
use crate::{actions, utils};
use crate::{errors::DomainError, AppData};

#[utoipa::path(
    get,
    path = "/api/v1/public/users/{user_id}",
    tag = "users",
    params(
        ("user_id" = String, Path, description = "User UUID"),
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found", body = ErrorResponseString),
    ),
)]
/// Finds user by UUID.
#[protect("RoleEnum::RoleAdmin", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(user_id))]
pub async fn get_user(
    app_data: web::Data<AppData>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    let uuid = UserUuid::from_str(&user_id.into_inner()).map_err(|err| {
        DomainError::new_bad_input_error(format!("Invalid UserUuid: {err}"))
    })?;
    let _ = tracing::info!("Getting user with uuid {uuid}");
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::find_user_by_uuid(&uuid, &mut conn)
    })
    .await??;
    let _ = tracing::debug!("{:?}", res);
    if let Some(user) = res {
        let _ = tracing::info!("Found user");
        Ok(HttpResponse::Ok().json(user))
    } else {
        let _ = tracing::warn!("Could not find user");
        let err = DomainError::new_entity_does_not_exist_error(format!(
            "No user found with uuid: {}",
            uuid
        ));
        Err(err)
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    tag = "users",
    params(
        ("page" = u16, Query, description = "Page number"),
        ("limit" = u16, Query, description = "Items per page"),
        ("q" = Option<String>, Query, description = "Search query"),
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
#[protect("RoleEnum::RoleAdmin", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pagination))]
pub async fn get_users(
    app_data: web::Data<AppData>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, DomainError> {
    let _ = tracing::info!("Users request");
    let users = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let p: Pagination = pagination.into_inner();
        if let Some(ref q) = p.q {
            actions::users::search_users(q.as_str(), &p, &mut conn)
        } else {
            actions::users::get_all_users(&p, &mut conn)
        }
    })
    .await??;

    let _ = tracing::info!("Found {} users", users.len());
    let _ = tracing::debug!("{:?}", users);

    Ok(HttpResponse::Ok().json(users))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/registration",
    tag = "users",
    request_body = NewUser,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Bad input", body = ErrorResponseString),
    ),
)]
// TODO rename to register user
/// Inserts a new user
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn add_user(
    app_data: web::Data<AppData>,
    form: web::Json<NewUser>,
) -> Result<HttpResponse, DomainError> {
    let new_user = form.into_inner();

    let email = new_user.email.clone();
    let email_for_tokio = email.clone();
    let mailer = app_data.mailer.clone();
    let pool_clone = app_data.pool.clone();
    let ttl_secs = app_data.config.email_token_ttl_verification_secs;

    let user = web::block(move || {
        let pool = &app_data.pool;
        let user_ids_cache = &app_data.user_ids_cache;
        let mut conn = pool.get()?;

        actions::users::insert_new_regular_user(
            new_user,
            app_data.config.hash_cost,
            user_ids_cache,
            &mut conn,
        )
    })
    .await??;

    let uid: i32 = user.id.as_uint() as i32;
    let user_name = user.username.as_str().to_string();

    tokio::spawn(async move {
        let token = tokens::generate_token();
        let thash = tokens::hash_token(&token);

        if let Ok(mut conn) = pool_clone.get() {
            use crate::schema::email_verification_tokens::dsl::*;
            if let Err(e) = diesel::insert_into(email_verification_tokens)
                .values((
                    user_id.eq(uid),
                    token_hash.eq(&thash),
                    expires_at.eq(chrono::Utc::now().naive_utc()
                        + chrono::Duration::seconds(ttl_secs as i64)),
                ))
                .execute(&mut conn)
            {
                tracing::error!(error = %e, uid = %uid, "Failed to store email verification token");
            }
        }

        if let Err(e) = mailer
            .send_verification_email(
                email_for_tokio.as_str(),
                &user_name,
                &token,
            )
            .await
        {
            tracing::error!(error = %e, "Failed to send verification email");
        }
    });

    let _ = tracing::info!("Created user with id={}", user.id);
    let _ = tracing::debug!("{:?}", user);

    Ok(HttpResponse::Created().json(user))
}

/// Request body for avatar upload.
#[allow(dead_code)]
#[derive(ToSchema)]
pub struct UploadAvatarRequest {
    /// Avatar image file (PNG, JPG, GIF, WebP).
    pub file: Vec<u8>,
}
#[utoipa::path(
    put,
    path = "/api/v1/avatars",
    tag = "users",
    request_body = UploadAvatarRequest,
    responses(
        (status = 200, description = "Avatar uploaded successfully", body = String),
        (status = 400, description = "Invalid file type or size", body = ErrorResponseString),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
/// Upload user avatar
#[tracing::instrument(level = "info", skip_all, fields(user_uuid))]
pub async fn upload_user_avatar(
    app_data: web::Data<AppData>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, DomainError> {
    // Get user UUID from header
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;

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

    let object_key = format!("avatars/{user_uuid}");
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

#[utoipa::path(
    delete,
    path = "/api/v1/avatars",
    tag = "users",
    responses(
        (status = 204, description = "Avatar deleted successfully"),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
/// Delete user avatar
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn delete_user_avatar(
    app_data: web::Data<AppData>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;

    let bucket = app_data.config.minio.bucket_name.clone();
    let minio_client = app_data.minio.client.clone();

    actions::users::delete_user_avatar(
        &user_uuid,
        &minior::Minio {
            client: minio_client,
        },
        &bucket,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/api/v1/public/avatars/{user_id}",
    tag = "users",
    params(
        ("user_id" = String, Path, description = "User UUID"),
    ),
    responses(
        (status = 200, description = "Avatar image"),
        (status = 404, description = "Avatar not found"),
    ),
)]
/// Get user avatar
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_user_avatar(
    app_data: web::Data<AppData>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    let uuid = UserUuid::from_str(&user_id.into_inner()).map_err(|err| {
        DomainError::new_bad_input_error(format!("Invalid UserUuid: {err}"))
    })?;
    let _ = tracing::info!("Getting avatar for user {uuid}");

    // Get the object from MinIO
    let object = app_data
        .minio
        .client
        .get_object()
        .bucket(&app_data.config.minio.bucket_name)
        .key(format!("avatars/{uuid}"))
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
    let stream = tokio_util::io::ReaderStream::new(reader);

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .streaming(stream))
}

#[utoipa::path(
    get,
    path = "/api/v1/user",
    tag = "users",
    responses(
        (status = 200, description = "User profile", body = User),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
/// Get the authenticated user's profile.
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn get_my_profile(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;

    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::find_active_user_by_uuid(&user_uuid, &mut conn)
    })
    .await??;

    match res {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => {
            let err = DomainError::new_entity_does_not_exist_error(
                "User not found".to_string(),
            );
            Err(err)
        }
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/user",
    tag = "users",
    request_body = UpdateUserProfile,
    responses(
        (status = 200, description = "Profile updated successfully", body = User),
        (status = 400, description = "Bad input", body = ErrorResponseString),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
/// Update the authenticated user's profile.
#[tracing::instrument(level = "info", skip_all, fields(form))]
pub async fn update_my_profile(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    form: web::Json<UpdateUserProfile>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;
    let has_email = form.0.email.is_some();

    if *form == UpdateUserProfile::default() {
        return Err(DomainError::new_bad_input_error(
            "At least one field must be provided for update".to_string(),
        ));
    }

    let email_update = form.0.email.clone();
    let mailer = app_data.mailer.clone();
    let pool_clone = app_data.pool.clone();
    let ttl_secs = app_data.config.email_token_ttl_verification_secs;
    let user = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::update_user_profile(&user_uuid, form.0, &mut conn)
    })
    .await??;

    if has_email {
        let uid: i32 = user.id.as_uint() as i32;
        let user_name = user.username.as_str().to_string();
        let email = email_update.unwrap();

        tokio::spawn(async move {
            let token = tokens::generate_token();
            let thash = tokens::hash_token(&token);

            if let Ok(mut conn) = pool_clone.get() {
                use crate::schema::email_verification_tokens::dsl::*;

                diesel::insert_into(email_verification_tokens)
                    .values((
                        user_id.eq(uid),
                        token_hash.eq(&thash),
                        expires_at.eq(
                            chrono::Utc::now().naive_utc()
                                + chrono::Duration::seconds(ttl_secs as i64),
                        ),
                    ))
                    .execute(&mut conn)
                    .map_err(|e| {
                        tracing::error!(error = %e, uid = %uid, "Failed to store email verification token");
                        e
                    })
                    .ok();
            }

            mailer.send_verification_email(
                email.as_str(),
                &user_name,
                &token,
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to send verification email");
                e
            })
            .ok();
        });
    }

    Ok(HttpResponse::Ok().json(user))
}

#[utoipa::path(
    delete,
    path = "/api/v1/user",
    tag = "users",
    responses(
        (status = 200, description = "Account deleted successfully"),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
/// Delete the authenticated user's account (soft delete).
/// Clears all sessions and avatar. Orphans associated jobs.
#[tracing::instrument(level = "info", skip_all, fields(user_uuid))]
pub async fn delete_my_account(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;

    let pool = app_data.pool.clone();
    web::block(move || {
        let mut conn = pool.get()?;
        actions::users::soft_delete_user(&user_uuid, &mut conn)
    })
    .await??;

    if let Err(e) = app_data
        .credentials_repo
        .delete_all_sessions(&user_uuid)
        .await
    {
        tracing::error!(error = %e, user_uuid = %user_uuid, "Failed to delete sessions during account deletion");
    }

    let bucket = app_data.config.minio.bucket_name.clone();
    let minio = minior::Minio {
        client: app_data.minio.client.clone(),
    };
    if let Err(e) =
        actions::users::delete_user_avatar(&user_uuid, &minio, &bucket).await
    {
        tracing::error!(error = %e, user_uuid = %user_uuid, "Failed to delete avatar during account deletion");
    }

    let cookie = Cookie::build("X-AUTH-TOKEN", "")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .expires(OffsetDateTime::UNIX_EPOCH)
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).finish())
}

#[utoipa::path(
    get,
    path = "/api/v1/public/profiles/{user_id}",
    tag = "users",
    params(
        ("user_id" = String, Path, description = "User UUID"),
    ),
    responses(
        (status = 200, description = "Public profile found", body = PublicProfile),
        (status = 404, description = "Profile not found", body = ErrorResponseString),
    ),
)]
/// Get a user's public profile.
#[tracing::instrument(level = "info", skip_all, fields(user_uuid))]
pub async fn get_public_profile(
    app_data: web::Data<AppData>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    let uuid = UserUuid::from_str(&user_id.into_inner()).map_err(|err| {
        DomainError::new_bad_input_error(format!("Invalid UserUuid: {err}"))
    })?;
    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::get_public_profile(&uuid, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
    get,
    path = "/api/v1/user/profile",
    tag = "users",
    responses(
        (status = 200, description = "Profile retrieved", body = PublicProfile),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
/// Get the authenticated user's profile.
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(user_uuid))]
pub async fn get_user_profile(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;

    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::get_profile(&user_uuid, &mut conn)
    })
    .await??;

    match res {
        Some(profile) => {
            Ok(HttpResponse::Ok().json(PublicProfile::from(&profile)))
        }
        None => Ok(HttpResponse::Ok().json(PublicProfile {
            user_uuid,
            bio: None,
            display_name: None,
            location: None,
            website_url: None,
            social_github: None,
            social_twitter: None,
        })),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/user/profile",
    tag = "users",
    request_body = CreateProfile,
    responses(
        (status = 201, description = "Profile created", body = PublicProfile),
        (status = 400, description = "Bad input", body = ErrorResponseString),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
/// Create the authenticated user's profile.
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(form))]
pub async fn create_user_profile(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    form: web::Json<CreateProfile>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;
    let create = form.into_inner();

    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::create_profile(&user_uuid, create, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Created().json(PublicProfile::from(&res)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/user/profile",
    tag = "users",
    request_body = UpdateProfile,
    responses(
        (status = 200, description = "Profile updated", body = PublicProfile),
        (status = 400, description = "Bad input", body = ErrorResponseString),
        (status = 401, description = "Missing or invalid auth token", body = ErrorResponseString),
    ),
)]
/// Update the authenticated user's profile.
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(form))]
pub async fn update_user_profile(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    form: web::Json<UpdateProfile>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = utils::extract_user_uuid_from_header(req.headers())?;
    let updates = form.into_inner();

    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::update_profile(&user_uuid, updates, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(PublicProfile::from(&res)))
}
