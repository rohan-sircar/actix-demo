use actix_web::{web, HttpRequest, HttpResponse};

use crate::actions;
use crate::actions::pet_profile_image_add;
use crate::models::pet_profile_images::{AddImageRequest, AddImagesRequest};
use crate::models::pet_profile_images::PetProfileImage;
use crate::models::pet_profile_insert::PetProfileInsertData;
use crate::models::pet_profile_update::PetProfileUpdateData;
use crate::models::pets::PetProfileUuid;
use crate::models::users::UserId;
use crate::{errors::DomainError, AppData};

/// Creates a new pet profile
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn add_pet_profile(
    app_data: web::Data<AppData>,
    form: web::Json<PetProfileInsertData>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    // Extract authenticated user ID from request headers
    let auth_user_id =
        crate::utils::extract_user_id_from_header(req.headers())?;
    let mut form_data = form.0;
    form_data.user_id = auth_user_id;

    let pet_profile = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profile_insert::create_pet_profile_from_insert_data(
            form_data, &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!(
        "Created pet profile with id: {}",
        pet_profile.basic_info.id
    );
    let _ = tracing::debug!("{:?}", pet_profile);

    Ok(HttpResponse::Created().json(pet_profile))
}

/// Gets a pet profile by UUID
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn get_pet_profile_for_pet_id(
    app_data: web::Data<AppData>,
    path: web::Path<PetProfileUuid>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    let pet_uuid = path.into_inner();
    let _ = tracing::info!("Getting pet profile with uuid {pet_uuid}");

    // Extract authenticated user ID from request headers
    let auth_user_id =
        crate::utils::extract_user_id_from_header(req.headers())?;

    // Clone pet_uuid before moving into the closure
    let pet_uuid_for_block = pet_uuid.clone();

    // Group all read operations into a single web::block call
    let result = web::block(move || {
        let mut conn = app_data.pool.get()?;
        
        // Check if the pet profile exists
        let exists = actions::pet_profile_full::check_pet_profile_exists_by_uuid(
            &pet_uuid_for_block,
            &mut conn,
        )?;

        if !exists {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "No pet profile found with uuid: {}",
                pet_uuid_for_block
            )));
        }

        // Check if the authenticated user owns this pet profile
        let is_owner = actions::pet_profile_full::check_pet_profile_ownership_by_uuid(
            &pet_uuid_for_block,
            &auth_user_id,
            &mut conn,
        )?;

        if !is_owner {
            return Err(DomainError::new_bad_input_error(format!(
                "You can only view your own pet profiles"
            )));
        }

        // Fetch the full pet profile
        let profile = actions::pet_profile_full::get_full_pet_profile_by_uuid(&pet_uuid_for_block, &mut conn)?;

        Ok(profile)
    })
    .await??;

    let _ = tracing::debug!("{:?}", result);

    if let Some(profile) = result {
        let _ = tracing::info!("Found pet profile");
        Ok(HttpResponse::Ok().json(profile))
    } else {
        let _ = tracing::warn!("Could not find pet profile");
        Err(DomainError::new_entity_does_not_exist_error(format!(
            "No pet profile found with uuid: {}",
            pet_uuid
        )))
    }
}

/// Gets all pet profiles for a user
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_pet_profiles_for_user(
    app_data: web::Data<AppData>,
    user_id: web::Path<UserId>,
) -> Result<HttpResponse, DomainError> {
    let user_id = user_id.into_inner();
    let _ = tracing::info!("Getting pet profiles for user {user_id}");

    let profiles = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profile_full::get_full_pet_profiles_for_user(
            &user_id, &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!("Found {} pet profiles", profiles.len());
    let _ = tracing::debug!("{:?}", profiles);

    Ok(HttpResponse::Ok().json(profiles))
}

/// Updates a pet profile by UUID
#[tracing::instrument(level = "info", skip(app_data, form))]
pub async fn update_pet_profile_for_pet_id(
    app_data: web::Data<AppData>,
    path: web::Path<PetProfileUuid>,
    form: web::Json<PetProfileUpdateData>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    let pet_uuid = path.into_inner();
    let pet_uuid_for_log = pet_uuid.clone();
    let update_data = form.0;

    let auth_user_id =
        crate::utils::extract_user_id_from_header(req.headers())?;

    // Group all read operations into a single web::block call
    let result = web::block(move || {
        let mut conn = app_data.pool.get()?;

        // Check if the pet profile exists
        let exists = actions::pet_profile_full::check_pet_profile_exists_by_uuid(
            &pet_uuid,
            &mut conn,
        )?;

        if !exists {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet profile with uuid {pet_uuid} does not exist"
            )));
        }

        // Check if the authenticated user owns this pet profile
        let is_owner = actions::pet_profile_full::check_pet_profile_ownership_by_uuid(
            &pet_uuid,
            &auth_user_id,
            &mut conn,
        )?;

        if !is_owner {
            return Err(DomainError::new_bad_input_error(format!(
                "You can only update your own pet profiles"
            )));
        }

        // Update the pet profile
        let updated_profile = actions::pet_profile_update::update_full_pet_profile(
            &pet_uuid,
            update_data,
            &mut conn,
        )?;

        Ok(updated_profile)
    })
    .await??;

    let _ = tracing::info!("Successfully updated pet profile with uuid {pet_uuid_for_log}");
    Ok(HttpResponse::Ok().json(result))
}

/// Deletes a pet profile by UUID
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn delete_pet_profile_for_pet_id(
    app_data: web::Data<AppData>,
    path: web::Path<PetProfileUuid>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    let pet_uuid = path.into_inner();
    let pet_uuid_for_log = pet_uuid.clone();

    // Extract authenticated user ID from request headers
    let auth_user_id =
        crate::utils::extract_user_id_from_header(req.headers())?;

    // Group all read operations and delete into a single web::block call
    let _ = web::block(move || {
        let mut conn = app_data.pool.get()?;

        // Check if the pet profile exists
        let exists = actions::pet_profile_full::check_pet_profile_exists_by_uuid(
            &pet_uuid,
            &mut conn,
        )?;

        if !exists {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet profile with uuid {pet_uuid} does not exist"
            )));
        }

        // Check if the authenticated user owns this pet profile
        let is_owner = actions::pet_profile_full::check_pet_profile_ownership_by_uuid(
            &pet_uuid,
            &auth_user_id,
            &mut conn,
        )?;

        if !is_owner {
            return Err(DomainError::new_bad_input_error(format!(
                "You can only delete your own pet profiles"
            )));
        }

        // Delete the pet profile
        actions::pet_profile_delete::delete_pet_profile(&pet_uuid, &mut conn)
    })
    .await??;

    let _ = tracing::info!("Successfully deleted pet profile with uuid {pet_uuid_for_log}");
    Ok(HttpResponse::NoContent().finish())
}

/// Deletes a specific pet profile image by image ID
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn delete_pet_profile_image(
    app_data: web::Data<AppData>,
    path: web::Path<(PetProfileUuid, i32)>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    let (pet_uuid, image_id) = path.into_inner();
    let pet_uuid_for_log = pet_uuid.clone();

    // Extract authenticated user ID from request headers
    let auth_user_id =
        crate::utils::extract_user_id_from_header(req.headers())?;

    // Group all read operations and delete into a single web::block call
    let deleted_image = web::block(move || {
        let mut conn = app_data.pool.get()?;

        // Check if the pet profile exists
        let exists = actions::pet_profile_full::check_pet_profile_exists_by_uuid(
            &pet_uuid,
            &mut conn,
        )?;

        if !exists {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet profile with uuid {pet_uuid} does not exist"
            )));
        }

        // Check if the authenticated user owns this pet profile
        let is_owner = actions::pet_profile_full::check_pet_profile_ownership_by_uuid(
            &pet_uuid,
            &auth_user_id,
            &mut conn,
        )?;

        if !is_owner {
            return Err(DomainError::new_bad_input_error(format!(
                "You can only delete images from your own pet profiles"
            )));
        }

        // Delete the pet profile image
        actions::pet_profile_image_delete::delete_pet_profile_image(
            &pet_uuid,
            image_id,
            &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!(
        "Successfully deleted pet profile image {image_id} for pet profile {pet_uuid_for_log}"
    );
    let _ = tracing::debug!("Deleted image: {:?}", deleted_image);
    Ok(HttpResponse::Ok().json(deleted_image))
}

/// Add a single image to a pet profile
#[tracing::instrument(level = "info", skip(app_data, req, form))]
pub async fn add_pet_profile_image(
    app_data: web::Data<AppData>,
    path: web::Path<PetProfileUuid>,
    form: web::Json<AddImageRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    let pet_uuid = path.into_inner();
    let image_url = form.image_url.clone();
    let is_primary = form.is_primary;
    let pet_uuid_for_log = pet_uuid.clone();

    // Extract authenticated user ID from request headers
    let auth_user_id =
        crate::utils::extract_user_id_from_header(req.headers())?;

    // Group all read operations and add into a single web::block call
    let added_image = web::block(move || {
        let mut conn = app_data.pool.get()?;

        // Check if the pet profile exists
        let exists = actions::pet_profile_full::check_pet_profile_exists_by_uuid(
            &pet_uuid,
            &mut conn,
        )?;

        if !exists {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet profile with uuid {pet_uuid} does not exist"
            )));
        }

        // Check ownership
        let is_owner = actions::pet_profile_full::check_pet_profile_ownership_by_uuid(
            &pet_uuid,
            &auth_user_id,
            &mut conn,
        )?;

        if !is_owner {
            return Err(DomainError::new_bad_input_error(format!(
                "You can only add images to your own pet profiles"
            )));
        }

        // Add the pet profile image
        pet_profile_image_add::add_pet_profile_image(
            &pet_uuid,
            image_url,
            is_primary,
            &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!(
            "Successfully added image {image_id} to pet profile {pet_uuid}",
            image_id = added_image.id,
            pet_uuid = pet_uuid_for_log
        );

    Ok(HttpResponse::Ok().json(added_image))
}

/// Add multiple images to a pet profile
#[tracing::instrument(level = "info", skip(app_data, req, form))]
pub async fn add_pet_profile_images(
    app_data: web::Data<AppData>,
    path: web::Path<PetProfileUuid>,
    form: web::Json<AddImagesRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    let pet_uuid = path.into_inner();
    let image_urls = form.image_urls.clone();
    let pet_uuid_for_log = pet_uuid.clone();

    // Extract authenticated user ID from request headers
    let auth_user_id =
        crate::utils::extract_user_id_from_header(req.headers())?;

    // Group all read operations and add into a single web::block call
    let added_images: Vec<PetProfileImage> = web::block(move || {
        let mut conn = app_data.pool.get()?;

        // Check if the pet profile exists
        let exists = actions::pet_profile_full::check_pet_profile_exists_by_uuid(
            &pet_uuid,
            &mut conn,
        )?;

        if !exists {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet profile with uuid {pet_uuid} does not exist"
            )));
        }

        // Check ownership
        let is_owner = actions::pet_profile_full::check_pet_profile_ownership_by_uuid(
            &pet_uuid,
            &auth_user_id,
            &mut conn,
        )?;

        if !is_owner {
            return Err(DomainError::new_bad_input_error(format!(
                "You can only add images to your own pet profiles"
            )));
        }

        // Add the pet profile images
        pet_profile_image_add::add_pet_profile_images(
            &pet_uuid,
            image_urls,
            &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!(
            "Successfully added {count} images to pet profile {pet_uuid}",
            count = added_images.len(),
            pet_uuid = pet_uuid_for_log
        );

    Ok(HttpResponse::Ok().json(added_images))
}

/// Set the primary image for a pet profile
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn set_primary_image(
    app_data: web::Data<AppData>,
    path: web::Path<(PetProfileUuid, i32)>,
    req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    let (pet_uuid, image_id) = path.into_inner();
    let pet_uuid_for_log = pet_uuid.clone();

    // Extract authenticated user ID from request headers
    let auth_user_id =
        crate::utils::extract_user_id_from_header(req.headers())?;

    // Group all read operations and set primary into a single web::block call
    let updated_image = web::block(move || {
        let mut conn = app_data.pool.get()?;

        // Check if the pet profile exists
        let exists = actions::pet_profile_full::check_pet_profile_exists_by_uuid(
            &pet_uuid,
            &mut conn,
        )?;

        if !exists {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet profile with uuid {pet_uuid} does not exist"
            )));
        }

        // Check ownership
        let is_owner = actions::pet_profile_full::check_pet_profile_ownership_by_uuid(
            &pet_uuid,
            &auth_user_id,
            &mut conn,
        )?;

        if !is_owner {
            return Err(DomainError::new_bad_input_error(format!(
                "You can only set primary images on your own pet profiles"
            )));
        }

        // Set the primary image
        pet_profile_image_add::set_primary_image(
            &pet_uuid,
            image_id,
            &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!(
        "Successfully set image {image_id} as primary for pet profile {pet_uuid}",
        image_id = updated_image.id,
        pet_uuid = pet_uuid_for_log
    );

    Ok(HttpResponse::Ok().json(updated_image))
}
