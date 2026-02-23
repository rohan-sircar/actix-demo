use actix_web::{web, HttpRequest, HttpResponse};

use crate::actions;
use crate::models::pets::PetProfileId;
use crate::models::pet_profile_insert::PetProfileInsertData;
use crate::models::users::UserId;
use crate::{errors::DomainError, AppData};

/// Creates a new pet profile
#[tracing::instrument(level = "info", skip(app_data))]
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

/// Gets a pet profile by pet ID
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_pet_profile_for_pet_id(
    app_data: web::Data<AppData>,
    path: web::Path<(UserId, PetProfileId)>,
) -> Result<HttpResponse, DomainError> {
    let (_, pet_id) = path.into_inner();
    let _ = tracing::info!("Getting pet profile with id {pet_id}");

    let pet_id2 = pet_id.clone();

    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profile_full::get_full_pet_profile(&pet_id2, &mut conn)
    })
    .await??;

    let _ = tracing::debug!("{:?}", res);

    if let Some(profile) = res {
        let _ = tracing::info!("Found pet profile");
        Ok(HttpResponse::Ok().json(profile))
    } else {
        let _ = tracing::warn!("Could not find pet profile");
        let err = DomainError::new_entity_does_not_exist_error(format!(
            "No pet profile found with id: {}",
            pet_id
        ));
        Err(err)
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
