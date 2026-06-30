use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use serde::Deserialize;

use crate::models::likes::{CreateLike, PetInteractionResponse};
use crate::models::misc::{PaginationLimit, PaginationOffset};
use crate::models::pets::{PetGender, PetSpecies};
use crate::models::roles::RoleEnum;
use crate::{errors::DomainError, AppData};

#[derive(Deserialize)]
pub struct DiscoverPetsQuery {
    pub species: Option<PetSpecies>,
    pub gender: Option<PetGender>,
    pub age_min: Option<f64>,
    pub age_max: Option<f64>,
    #[serde(default = "crate::models::misc::default_limit")]
    pub limit: PaginationLimit,
    #[serde(default = "crate::models::misc::default_offset")]
    pub offset: PaginationOffset,
}

/// Returns a single random pet that the user has not yet interacted with.
#[utoipa::path(
    get,
    path = "/api/v1/private/discover/next",
    tag = "discover",
    responses(
        (status = 200, description = "Next pet for discovery", body = crate::models::pets::PublicPet),
        (status = 204, description = "No more pets available"),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn discover_next(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    let pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::discover::get_next_pet(&user_id, &mut conn)
    })
    .await??;

    match pet {
        Some(pet) => Ok(HttpResponse::Ok().json(pet)),
        None => Ok(HttpResponse::NoContent().finish()),
    }
}

/// Returns a paginated list of pets with optional filters.
#[utoipa::path(
    get,
    path = "/api/v1/private/discover/pets",
    tag = "discover",
    params(
        ("species" = Option<PetSpecies>, Query, description = "Filter by species"),
        ("gender" = Option<PetGender>, Query, description = "Filter by gender (male/female/unspecified)"),
        ("age_min" = Option<f64>, Query, description = "Minimum age in years"),
        ("age_max" = Option<f64>, Query, description = "Maximum age in years"),
        ("limit" = PaginationLimit, Query, description = "Results per page (max 50)"),
        ("offset" = PaginationOffset, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "Paginated list of pets", body = Vec<PublicPet>),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn discover_pets(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    query: web::Query<DiscoverPetsQuery>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    // let page = query.page.unwrap_or(1);
    // let per_page = query.per_page.unwrap_or(20);

    let (pets, total_count) = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        // let p = page.max(1);
        // let pp = per_page.min(50);
        // let offset = (p - 1) * pp;

        crate::actions::discover::list_discoverable_pets(
            &user_id, &query, &mut conn,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "pets": pets,
        "total_count": total_count,
    })))
}

/// Creates a like or dislike for a pet. Detects mutual matches.
#[utoipa::path(
    post,
    path = "/api/v1/private/likes",
    tag = "likes",
    request_body = CreateLike,
    responses(
        (status = 201, description = "Like created", body = crate::models::likes::LikeResponse),
        (status = 400, description = "Bad input", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
        (status = 409, description = "Already liked this pet", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(form))]
pub async fn create_like(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    form: web::Json<CreateLike>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    let like_response = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::likes::create_like(
            &user_id,
            form.into_inner(),
            &mut conn,
        )
    })
    .await??;

    Ok(HttpResponse::Created().json(like_response))
}

/// Returns the list of pets the user has liked (outgoing likes).
#[utoipa::path(
    get,
    path = "/api/v1/private/likes/sent",
    tag = "likes",
    responses(
        (status = 200, description = "List of sent likes", body = Vec<crate::models::likes::LikeWithPet>),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_likes_sent(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    let result = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::likes::list_likes_sent(&user_id, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

/// Returns the list of pets whose owners were liked by others (incoming likes).
#[utoipa::path(
    get,
    path = "/api/v1/private/likes/received",
    tag = "likes",
    responses(
        (status = 200, description = "List of received likes", body = Vec<crate::models::likes::LikeWithPet>),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_likes_received(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    let result = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::likes::list_likes_received(&user_id, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

/// Returns a stub response for messages.
#[utoipa::path(
    post,
    path = "/api/v1/private/messages",
    tag = "discover",
    responses(
        (status = 501, description = "Not yet implemented"),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn stub_messages(
    _req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "stub"
    })))
}

/// Returns the list of mutual matches with both pets involved for the current user.
#[utoipa::path(
    get,
    path = "/api/v1/private/matches-with-pets",
    tag = "matches",
    responses(
        (status = 200, description = "List of mutual matches with both pets", body = Vec<crate::models::likes::MatchWithPets>),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_matches_with_pets(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    let result = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let user_id =
            crate::actions::users::get_user_id_by_uuid(&user_uuid, &mut conn)?;
        crate::actions::likes::list_matches_with_pets(&user_id, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

/// Checks if the current user has already interacted with a specific pet.
#[utoipa::path(
    get,
    path = "/api/v1/private/likes/check/{pet_uuid}",
    tag = "likes",
    responses(
        (status = 200, description = "Interaction status", body = PetInteractionResponse),
        (status = 401, description = "Missing auth", body = DomainError),
        (status = 404, description = "Pet not found", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn check_pet_interaction(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    path: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let pet_uuid_str = path.into_inner();

    let result =
        web::block(move || -> Result<PetInteractionResponse, DomainError> {
            let pool = &app_data.pool;
            let mut conn = pool.get()?;
            let user_id = crate::actions::users::get_user_id_by_uuid(
                &user_uuid, &mut conn,
            )?;
            let pet_uuid = crate::models::pets::PetUuid::try_from(pet_uuid_str)
                .map_err(|e| {
                    DomainError::new_internal_error(format!(
                        "Invalid pet UUID: {}",
                        e
                    ))
                })?;
            let interaction = crate::actions::likes::get_pet_interaction(
                &user_id, &pet_uuid, &mut conn,
            )?;

            Ok(interaction)
        })
        .await??;

    Ok(HttpResponse::Ok().json(result))
}

/// Returns a stub response for reports.
#[utoipa::path(
    post,
    path = "/api/v1/private/reports",
    tag = "discover",
    responses(
        (status = 501, description = "Not yet implemented"),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn stub_reports(
    _req: HttpRequest,
) -> Result<HttpResponse, DomainError> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "stub"
    })))
}
