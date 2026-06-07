use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use serde::Deserialize;

use crate::models::pets::{CreatePet, UpdatePet};
use crate::models::roles::RoleEnum;
use crate::{errors::DomainError, AppData};

#[derive(Deserialize)]
pub(crate) struct ListPetsQuery {
    species: Option<String>,
    traits: Option<String>,
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
    path = "/api/public/pets/{pet_id}",
    tag = "pets",
    params(
        ("pet_id" = crate::models::pets::PetId, Path, description = "Pet ID"),
    ),
    responses(
        (status = 200, description = "Pet found", body = crate::models::pets::PublicPet),
        (status = 404, description = "Pet not found", body = DomainError),
    ),
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_public_pet(
    app_data: web::Data<AppData>,
    pet_id: web::Path<crate::models::pets::PetId>,
) -> Result<HttpResponse, DomainError> {
    let pet_id = pet_id.into_inner();

    let pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::pets::get_public_pet(&pet_id, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(pet))
}

#[utoipa::path(
    post,
    path = "/api/user/pets",
    tag = "pets",
    request_body = CreatePet,
    responses(
        (status = 201, description = "Pet created successfully", body = crate::models::pets::PublicPet),
        (status = 400, description = "Bad input", body = DomainError),
        (status = 401, description = "Missing or invalid auth token", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(form))]
pub async fn create_pet(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    form: web::Json<CreatePet>,
) -> Result<HttpResponse, DomainError> {
    let user_id = crate::utils::extract_user_id_from_header(req.headers())?;

    let public_pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
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
        ("traits" = Option<String>, Query, description = "Filter by trait names (comma-separated)"),
    ),
    responses(
        (status = 200, description = "List of user's pets", body = Vec<crate::models::pets::PublicPet>),
        (status = 401, description = "Missing or invalid auth token", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_pets(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    query: web::Query<ListPetsQuery>,
) -> Result<HttpResponse, DomainError> {
    let user_id = crate::utils::extract_user_id_from_header(req.headers())?;

    let pets = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;

        let species_filter = query.species.as_deref();

        let trait_filter = query.traits.as_ref().and_then(|t| {
            if t.is_empty() {
                None
            } else {
                Some(t.split(',').map(|s| s.trim()).collect::<Vec<&str>>())
            }
        });

        crate::actions::pets::list_pets(
            &user_id,
            species_filter,
            trait_filter,
            &mut conn,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(pets))
}

#[utoipa::path(
    get,
    path = "/api/user/pets/{pet_id}",
    tag = "pets",
    params(
        ("pet_id" = crate::models::pets::PetId, Path, description = "Pet ID"),
    ),
    responses(
        (status = 200, description = "Pet found", body = crate::models::pets::PublicPet),
        (status = 404, description = "Pet not found", body = DomainError),
        (status = 401, description = "Missing or invalid auth token", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_pet(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    pet_id: web::Path<crate::models::pets::PetId>,
) -> Result<HttpResponse, DomainError> {
    let user_id = crate::utils::extract_user_id_from_header(req.headers())?;
    let pet_id = pet_id.into_inner();

    let public_pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::pets::get_pet(&pet_id, &user_id, &mut conn)
    })
    .await??;

    let public_pet = match public_pet {
        Some(p) => p,
        None => {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet {} not found or does not belong to user {}",
                pet_id, user_id
            )))
        }
    };

    Ok(HttpResponse::Ok().json(public_pet))
}

#[utoipa::path(
    patch,
    path = "/api/user/pets/{pet_id}",
    tag = "pets",
    params(
        ("pet_id" = crate::models::pets::PetId, Path, description = "Pet ID"),
    ),
    request_body = UpdatePet,
    responses(
        (status = 200, description = "Pet updated successfully", body = crate::models::pets::PublicPet),
        (status = 400, description = "Bad input", body = DomainError),
        (status = 404, description = "Pet not found", body = DomainError),
        (status = 401, description = "Missing or invalid auth token", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(form))]
pub async fn update_pet(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    pet_id: web::Path<crate::models::pets::PetId>,
    form: web::Json<UpdatePet>,
) -> Result<HttpResponse, DomainError> {
    let user_id = crate::utils::extract_user_id_from_header(req.headers())?;
    let pet_id = pet_id.into_inner();

    let public_pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::pets::update_pet(
            &pet_id,
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
    path = "/api/user/pets/{pet_id}",
    tag = "pets",
    params(
        ("pet_id" = crate::models::pets::PetId, Path, description = "Pet ID"),
    ),
    responses(
        (status = 200, description = "Pet deleted successfully"),
        (status = 404, description = "Pet not found", body = DomainError),
        (status = 401, description = "Missing or invalid auth token", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn delete_pet(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    pet_id: web::Path<crate::models::pets::PetId>,
) -> Result<HttpResponse, DomainError> {
    let user_id = crate::utils::extract_user_id_from_header(req.headers())?;
    let pet_id = pet_id.into_inner();

    web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::pets::delete_pet(&pet_id, &user_id, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().finish())
}
