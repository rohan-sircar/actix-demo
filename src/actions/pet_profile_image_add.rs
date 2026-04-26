use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable};
use diesel::QueryableByName;

use crate::errors::DomainError;
use crate::models::pet_profile_images::{NewPetProfileImage, PetProfileImage};
use crate::models::pets::{PetProfileImageUuid, PetProfileUuid};
use crate::schema::pet_profile_images;
use crate::types::DbConnection;

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SortOrderResult {
    #[diesel(sql_type = Nullable<Integer>)]
    sort_order: Option<i32>,
}

/// Add a single image to an existing pet profile
pub fn add_pet_profile_image(
    pet_uuid: &PetProfileUuid,
    image_url: String,
    is_primary: Option<bool>,
    conn: &mut DbConnection,
) -> Result<PetProfileImage, DomainError> {
    let _ = tracing::info!(
        "Adding image to pet profile {pet_uuid}: {image_url:?}"
    );

    // Determine sort_order - use max existing + 1, or 0 if no images exist
    let max_sort_order: i32 = diesel::sql_query(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 as sort_order FROM pet_profile_images WHERE pet_profile_uuid = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(pet_uuid)
    .get_result::<SortOrderResult>(conn)
    .map_err(|err| {
        DomainError::new_internal_error(format!(
            "Failed to determine sort order: {err}"
        ))
    })?
    .sort_order
    .unwrap_or(1);

    let new_image = NewPetProfileImage {
        pet_profile_uuid: pet_uuid.clone(),
        image_url: image_url.clone(),
        is_primary,
        sort_order: Some(max_sort_order),
    };

    // Insert the new image
    diesel::insert_into(pet_profile_images::table)
        .values(new_image)
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to insert pet profile image: {err}"
            ))
        })
}

/// Add multiple images to an existing pet profile
pub fn add_pet_profile_images(
    pet_uuid: &PetProfileUuid,
    image_urls: Vec<String>,
    conn: &mut DbConnection,
) -> Result<Vec<PetProfileImage>, DomainError> {
    let _ = tracing::info!(
        "Adding {count} images to pet profile {pet_uuid}",
        count = image_urls.len()
    );

    // Get existing sort order max
    let base_sort_order: i32 = diesel::sql_query(
        "SELECT COALESCE(MAX(sort_order), 0) as sort_order FROM pet_profile_images WHERE pet_profile_uuid = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(pet_uuid)
    .get_result::<SortOrderResult>(conn)
    .map_err(|err| {
        DomainError::new_internal_error(format!(
            "Failed to determine sort order: {err}"
        ))
    })?
    .sort_order
    .unwrap_or(0);

    let mut inserted_images = Vec::new();

    for (index, url) in image_urls.into_iter().enumerate() {
        let new_image = NewPetProfileImage {
            pet_profile_uuid: pet_uuid.clone(),
            image_url: url.clone(),
            is_primary: None,
            sort_order: Some(base_sort_order + index as i32 + 1),
        };

        let inserted = diesel::insert_into(pet_profile_images::table)
            .values(new_image)
            .get_result(conn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to insert pet profile image: {err}"
                ))
            })?;

        inserted_images.push(inserted);
    }

    Ok(inserted_images)
}

/// Set a specific image as primary for a pet profile by UUID
pub fn set_primary_image_by_uuid(
    pet_uuid: &PetProfileUuid,
    image_uuid: &PetProfileImageUuid,
    conn: &mut DbConnection,
) -> Result<PetProfileImage, DomainError> {
    let _ = tracing::info!(
        "Setting image {image_uuid} as primary for pet profile {pet_uuid}"
    );

    use crate::schema::pet_profile_images::dsl::*;

    // First, unset all other images as primary
    diesel::update(pet_profile_images.filter(pet_profile_uuid.eq(pet_uuid)))
        .set(is_primary.eq::<Option<bool>>(None))
        .execute(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to unset primary images: {err}"
            ))
        })?;

    // Then set the specified image as primary
    diesel::update(pet_profile_images.filter(uuid.eq(image_uuid)))
        .filter(pet_profile_uuid.eq(pet_uuid))
        .set(is_primary.eq::<Option<bool>>(Some(true)))
        .get_result(conn)
        .map_err(|err| {
            match err {
                diesel::result::Error::NotFound => {
                    DomainError::new_entity_does_not_exist_error(format!(
                        "Pet profile image {image_uuid} not found for pet profile {pet_uuid}"
                    ))
                }
                _ => DomainError::new_internal_error(format!(
                    "Failed to set primary image: {err}"
                )),
            }
        })
}
