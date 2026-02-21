use actix_web::{web, HttpResponse};

use crate::actions;
use crate::models::pet_basic_info::PetBasicInfoId;
use crate::models::pet_profile_insert::PetProfileInsertData;
use crate::models::users::UserId;
use crate::{errors::DomainError, AppData};

/// Creates a new pet profile
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn add_pet_profile(
    app_data: web::Data<AppData>,
    form: web::Json<PetProfileInsertData>,
) -> Result<HttpResponse, DomainError> {
    let pet_profile = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profiles::create_pet_basic_info(
            form.0.to_new_pet_basic_info(),
            &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!("Created pet profile with id: {}", pet_profile.id);
    let _ = tracing::debug!("{:?}", pet_profile);

    Ok(HttpResponse::Created().json(pet_profile))
}

/// Gets a pet profile by pet ID
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_pet_profile_for_pet_id(
    app_data: web::Data<AppData>,
    pet_id: web::Path<PetBasicInfoId>,
) -> Result<HttpResponse, DomainError> {
    let pet_id = pet_id.into_inner();
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
