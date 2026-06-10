use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use diesel::prelude::*;
use serde::Deserialize;

use crate::models::pets::{CreatePet, PublicPetImage, UpdatePet};
use crate::models::roles::RoleEnum;
use crate::{errors::DomainError, AppData};

#[derive(Deserialize)]
pub(crate) struct ListPetsQuery {
    species: Option<String>,
    traits: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct PetImagePath {
    pet_uuid: crate::models::pets::PetUuid,
    image_uuid: uuid::Uuid,
}

#[derive(Deserialize)]
pub(crate) struct PublicImageVariantPath {
    image_uuid: uuid::Uuid,
    variant: String,
}

#[utoipa::path(
    get,
    path = "/api/public/pets/traits",
    tag = "pets",
    responses(
        (status = 200, description = "List of personality traits", body = Vec<crate::models::pets::PersonalityTrait>),
    ),
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_traits(
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let traits = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::pets::list_traits(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(traits))
}

#[utoipa::path(
    get,
    path = "/api/public/pets/{pet_uuid}",
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
    ),
    responses(
        (status = 200, description = "Pet found", body = crate::models::pets::PublicPet),
        (status = 404, description = "Pet not found", body = DomainError),
    ),
)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid))]
pub async fn get_public_pet(
    app_data: web::Data<AppData>,
    pet_uuid: web::Path<crate::models::pets::PetUuid>,
) -> Result<HttpResponse, DomainError> {
    let pet_uuid = pet_uuid.into_inner();

    let pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::pets::get_public_pet(&pet_uuid, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(pet))
}

#[utoipa::path(
    post,
    path = "/api/user/pets",
    tag = "pets",
    request_body = crate::models::pets::CreatePet,
    responses(
        (status = 201, description = "Pet created successfully", body = crate::models::pets::PublicPet),
        (status = 400, description = "Bad input", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(form))]
pub async fn create_pet(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    form: web::Json<CreatePet>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    let public_pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::create_pet(&user_id, form.into_inner(), &mut conn)
    })
    .await??;

    Ok(HttpResponse::Created().json(public_pet))
}

#[utoipa::path(
    get,
    path = "/api/user/pets",
    tag = "pets",
    params(
        ("species" = Option<String>, Query, description = "Filter by species"),
        ("traits" = Option<String>, Query, description = "Filter by traits"),
    ),
    responses(
        (status = 200, description = "List of user's pets", body = Vec<crate::models::pets::PublicPet>),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_pets(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    query: web::Query<ListPetsQuery>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    let pets = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::list_pets(
            &user_id,
            query.species.as_deref(),
            query
                .traits
                .as_ref()
                .map(|t| t.split(',').collect::<Vec<_>>()),
            &mut conn,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(pets))
}

#[utoipa::path(
    get,
    path = "/api/user/pets/{pet_uuid}",
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
    ),
    responses(
        (status = 200, description = "Pet found", body = crate::models::pets::PublicPet),
        (status = 404, description = "Pet not found", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid))]
pub async fn get_pet(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    pet_uuid: web::Path<crate::models::pets::PetUuid>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let pet_uuid = pet_uuid.into_inner();

    let public_pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::get_pet(&pet_uuid, &user_id, &mut conn)
    })
    .await??;

    match public_pet {
        Some(pet) => Ok(HttpResponse::Ok().json(pet)),
        None => Err(DomainError::new_entity_does_not_exist_error(format!(
            "Pet {} not found",
            pet_uuid
        ))),
    }
}

#[utoipa::path(
    patch,
    path = "/api/user/pets/{pet_uuid}",
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
    ),
    request_body = crate::models::pets::UpdatePet,
    responses(
        (status = 200, description = "Pet updated successfully", body = crate::models::pets::PublicPet),
        (status = 400, description = "Bad input", body = DomainError),
        (status = 404, description = "Pet not found", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid, form))]
pub async fn update_pet(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    pet_uuid: web::Path<crate::models::pets::PetUuid>,
    form: web::Json<UpdatePet>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let pet_uuid = pet_uuid.into_inner();

    let public_pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::update_pet(
            &pet_uuid,
            &user_id,
            form.into_inner(),
            &mut conn,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(public_pet))
}

#[utoipa::path(
    delete,
    path = "/api/user/pets/{pet_uuid}",
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
    ),
    responses(
        (status = 200, description = "Pet deleted successfully"),
        (status = 404, description = "Pet not found", body = DomainError),
        (status = 401, description = "Missing or invalid auth token", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid))]
pub async fn delete_pet(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    pet_uuid: web::Path<crate::models::pets::PetUuid>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let pet_uuid = pet_uuid.into_inner();

    web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::delete_pet(&pet_uuid, &user_id, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    post,
    path = "/api/user/pets/{pet_uuid}/images",
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
    ),
    request_body = Vec<u8>,
    responses(
        (status = 201, description = "Image uploaded", body = crate::models::pets::PublicPetImage),
        (status = 400, description = "Bad input", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
        (status = 404, description = "Not found", body = DomainError),
        (status = 413, description = "File too large", body = DomainError),
        (status = 415, description = "Unsupported media type", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid))]
pub async fn upload_pet_image(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    pet_uuid: web::Path<crate::models::pets::PetUuid>,
    payload: web::Payload,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let content_type =
        crate::utils::extract_header_value(req.headers(), "content-type")?;
    let pet_uuid = pet_uuid.into_inner();

    let image_bytes = crate::utils::validate_image_stream(
        payload,
        &content_type,
        app_data.config.minio.max_pet_image_size_bytes as usize,
    )
    .await?;

    let minio_client = app_data.minio.client.clone();
    let bucket_name = app_data.config.minio.bucket_name.clone();
    let bucket_name_for_action = bucket_name.clone();

    let (public_image, thumbnail, medium, original) = web::block(move || {
        let mut conn = app_data.pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::upload_pet_image(
            &pet_uuid,
            &user_id,
            image_bytes.to_vec(),
            &app_data.pool,
            &bucket_name_for_action,
        )
    })
    .await??;

    let thumbnail_key = format!("pets/{}/thumbnail.webp", pet_uuid);
    let medium_key = format!("pets/{}/medium.webp", pet_uuid);
    let original_key = format!("pets/{}/original.webp", pet_uuid);

    minio_client
        .put_object()
        .bucket(&bucket_name)
        .key(&thumbnail_key)
        .body(thumbnail.into())
        .content_type("image/webp")
        .send()
        .await
        .map_err(|e| {
            DomainError::new_file_upload_failed(format!(
                "Thumbnail upload failed: {}",
                e
            ))
        })?;

    minio_client
        .put_object()
        .bucket(&bucket_name)
        .key(&medium_key)
        .body(medium.into())
        .content_type("image/webp")
        .send()
        .await
        .map_err(|e| {
            DomainError::new_file_upload_failed(format!(
                "Medium upload failed: {}",
                e
            ))
        })?;

    minio_client
        .put_object()
        .bucket(&bucket_name)
        .key(&original_key)
        .body(original.into())
        .content_type("image/webp")
        .send()
        .await
        .map_err(|e| {
            DomainError::new_file_upload_failed(format!(
                "Original upload failed: {}",
                e
            ))
        })?;

    Ok(HttpResponse::Created().json(public_image))
}

#[utoipa::path(
    get,
    path = "/api/user/pets/{pet_uuid}/images",
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
    ),
    responses(
        (status = 200, description = "List of pet images", body = Vec<crate::models::pets::PublicPetImage>),
        (status = 404, description = "Pet not found", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid))]
pub async fn list_pet_images(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    pet_uuid: web::Path<crate::models::pets::PetUuid>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let pet_uuid = pet_uuid.into_inner();

    let images = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::list_pet_images(&pet_uuid, &user_id, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(images))
}

#[utoipa::path(
    delete,
    path = "/api/user/pets/{pet_uuid}/images/{image_uuid}",
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
        ("image_uuid" = uuid::Uuid, Path, description = "Image UUID"),
    ),
    responses(
        (status = 200, description = "Image deleted successfully"),
        (status = 404, description = "Not found", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid, image_uuid))]
pub async fn delete_pet_image(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    path: web::Path<PetImagePath>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let PetImagePath {
        pet_uuid,
        image_uuid,
    } = path.into_inner();

    web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::delete_pet_image(
            &pet_uuid,
            &image_uuid,
            &user_id,
            &mut conn,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    patch,
    path = "/api/user/pets/{pet_uuid}/images/{image_uuid}",
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
        ("image_uuid" = uuid::Uuid, Path, description = "Image UUID"),
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Image updated", body = crate::models::pets::PublicPetImage),
        (status = 400, description = "Bad input", body = DomainError),
        (status = 404, description = "Not found", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid, image_uuid))]
pub async fn set_primary_pet_image(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    path: web::Path<PetImagePath>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let PetImagePath {
        pet_uuid,
        image_uuid,
    } = path.into_inner();

    if body.get("is_primary").and_then(|v| v.as_bool()) != Some(true) {
        return Err(DomainError::new_bad_input_error(
            "is_primary must be true".to_string(),
        ));
    }

    let public_image = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::pets::set_primary_image(
            &pet_uuid,
            &image_uuid,
            &user_id,
            &mut conn,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(public_image))
}

#[utoipa::path(
    get,
    path = "/api/public/pets/images/{image_uuid}",
    tag = "pets",
    params(
        ("image_uuid" = uuid::Uuid, Path, description = "Image UUID"),
    ),
    responses(
        (status = 200, description = "Image metadata", body = crate::models::pets::PublicPetImage),
        (status = 404, description = "Image not found", body = DomainError),
    ),
)]
#[tracing::instrument(level = "info", skip_all, fields(image_uuid))]
pub async fn get_public_pet_image(
    app_data: web::Data<AppData>,
    image_uuid: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, DomainError> {
    let image_uuid = image_uuid.into_inner();

    let image = web::block(move || -> Result<Option<crate::models::pets::PetImage>, crate::errors::DomainError> {
        let pool = &app_data.pool;
        let mut conn = pool.get().map_err(|e| {
            crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))
        })?;
        use crate::schema::pet_images::dsl as pet_images;
        let result: Result<crate::models::pets::PetImage, _> = pet_images::pet_images
            .filter(pet_images::uuid.eq(image_uuid))
            .first(&mut conn);
        match result {
            Ok(img) => Ok(Some(img)),
            Err(diesel::NotFound) => Ok(None),
            Err(e) => Err(crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))),
        }
    })
    .await??;

    let image = match image {
        Some(img) => img,
        None => {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Image {} not found",
                image_uuid
            )))
        }
    };

    Ok(HttpResponse::Ok().json(PublicPetImage::from(&image)))
}

#[utoipa::path(
    get,
    path = "/api/public/pets/images/{image_uuid}/{variant}",
    tag = "pets",
    params(
        ("image_uuid" = uuid::Uuid, Path, description = "Image UUID"),
        ("variant" = String, Path, description = "Variant (thumbnail, medium, original)"),
    ),
    responses(
        (status = 200, description = "Image stream"),
        (status = 404, description = "Image not found", body = DomainError),
    ),
)]
#[tracing::instrument(level = "info", skip_all, fields(image_uuid, variant))]
pub async fn get_pet_image_variant(
    app_data: web::Data<AppData>,
    path: web::Path<PublicImageVariantPath>,
) -> Result<HttpResponse, DomainError> {
    let PublicImageVariantPath {
        image_uuid,
        variant,
    } = path.into_inner();
    let app_data_clone = app_data.clone();

    let (object_key, content_type) = match variant.as_str() {
        "thumbnail" => {
            let image = web::block(move || -> Result<Option<crate::models::pets::PetImage>, crate::errors::DomainError> {
                let pool = &app_data_clone.pool;
                let mut conn = pool.get().map_err(|e| {
                    crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))
                })?;
                use crate::schema::pet_images::dsl as pet_images;
                let result: Result<crate::models::pets::PetImage, _> = pet_images::pet_images
                    .filter(pet_images::uuid.eq(image_uuid))
                    .first(&mut conn);
                match result {
                    Ok(img) => Ok(Some(img)),
                    Err(diesel::NotFound) => Ok(None),
                    Err(e) => Err(crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))),
                }
            })
            .await??;

            let image = match image {
                Some(img) => img,
                None => {
                    return Err(DomainError::new_entity_does_not_exist_error(
                        format!("Image {} not found", image_uuid),
                    ))
                }
            };

            (image.thumbnail_key, "image/webp")
        }
        "medium" => {
            let image = web::block(move || -> Result<Option<crate::models::pets::PetImage>, crate::errors::DomainError> {
                let pool = &app_data_clone.pool;
                let mut conn = pool.get().map_err(|e| {
                    crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))
                })?;
                use crate::schema::pet_images::dsl as pet_images;
                let result: Result<crate::models::pets::PetImage, _> = pet_images::pet_images
                    .filter(pet_images::uuid.eq(image_uuid))
                    .first(&mut conn);
                match result {
                    Ok(img) => Ok(Some(img)),
                    Err(diesel::NotFound) => Ok(None),
                    Err(e) => Err(crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))),
                }
            })
            .await??;

            let image = match image {
                Some(img) => img,
                None => {
                    return Err(DomainError::new_entity_does_not_exist_error(
                        format!("Image {} not found", image_uuid),
                    ))
                }
            };

            (image.medium_key, "image/webp")
        }
        "original" => {
            let image = web::block(move || -> Result<Option<crate::models::pets::PetImage>, crate::errors::DomainError> {
                let pool = &app_data_clone.pool;
                let mut conn = pool.get().map_err(|e| {
                    crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))
                })?;
                use crate::schema::pet_images::dsl as pet_images;
                let result: Result<crate::models::pets::PetImage, _> = pet_images::pet_images
                    .filter(pet_images::uuid.eq(image_uuid))
                    .first(&mut conn);
                match result {
                    Ok(img) => Ok(Some(img)),
                    Err(diesel::NotFound) => Ok(None),
                    Err(e) => Err(crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))),
                }
            })
            .await??;

            let image = match image {
                Some(img) => img,
                None => {
                    return Err(DomainError::new_entity_does_not_exist_error(
                        format!("Image {} not found", image_uuid),
                    ))
                }
            };

            (image.original_key, "image/webp")
        }
        _ => {
            return Err(DomainError::new_bad_input_error(format!(
                "Invalid variant: {}",
                variant
            )))
        }
    };

    let object = app_data
        .minio
        .client
        .get_object()
        .bucket(&app_data.config.minio.bucket_name)
        .key(&object_key)
        .send()
        .await
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to get object: {}",
                err
            ))
        })?;

    let reader = object.body.into_async_read();
    let stream = tokio_util::io::ReaderStream::new(reader);

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .streaming(stream))
}
