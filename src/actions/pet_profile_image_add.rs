use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable};
use diesel::QueryableByName;

use crate::errors::DomainError;
use crate::models::pet_profile_images::{NewPetProfileImage, PetProfileImage};
use crate::models::pets::PetProfileId;
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
    pet_id: &PetProfileId,
    image_url: String,
    is_primary: Option<bool>,
    conn: &mut DbConnection,
) -> Result<PetProfileImage, DomainError> {
    let _ = tracing::info!(
        "Adding image to pet profile {pet_id}: {image_url:?}"
    );

    // Determine sort_order - use max existing + 1, or 0 if no images exist
    let max_sort_order: i32 = diesel::sql_query(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 as sort_order FROM pet_profile_images WHERE pet_profile_id = $1"
    )
    .bind::<Integer, _>(pet_id.as_i32())
    .get_result::<SortOrderResult>(conn)
    .map_err(|err| {
        DomainError::new_internal_error(format!(
            "Failed to determine sort order: {err}"
        ))
    })?
    .sort_order
    .unwrap_or(1);

    let new_image = NewPetProfileImage {
        pet_profile_id: pet_id.clone(),
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
    pet_id: &PetProfileId,
    image_urls: Vec<String>,
    conn: &mut DbConnection,
) -> Result<Vec<PetProfileImage>, DomainError> {
    let _ = tracing::info!(
        "Adding {count} images to pet profile {pet_id}",
        count = image_urls.len()
    );

    // Get existing sort order max
    let base_sort_order: i32 = diesel::sql_query(
        "SELECT COALESCE(MAX(sort_order), 0) as sort_order FROM pet_profile_images WHERE pet_profile_id = $1"
    )
    .bind::<Integer, _>(pet_id.as_i32())
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
            pet_profile_id: pet_id.clone(),
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

/// Set the primary image for a pet profile
pub fn set_primary_image(
    pet_id: &PetProfileId,
    image_id: i32,
    conn: &mut DbConnection,
) -> Result<PetProfileImage, DomainError> {
    let _ = tracing::info!(
        "Setting image {image_id} as primary for pet profile {pet_id}"
    );

    // First, unset all other images as primary
    diesel::update(
        pet_profile_images::table
            .filter(pet_profile_images::pet_profile_id.eq(pet_id.as_i32())),
    )
    .set(pet_profile_images::is_primary.eq::<Option<bool>>(None))
    .execute(conn)
    .map_err(|err| {
        DomainError::new_internal_error(format!(
            "Failed to unset primary images: {err}"
        ))
    })?;

    // Then set the specified image as primary
    diesel::update(pet_profile_images::table.find(image_id))
        .filter(pet_profile_images::pet_profile_id.eq(pet_id.as_i32()))
        .set(pet_profile_images::is_primary.eq::<Option<bool>>(Some(true)))
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to set primary image: {err}"
            ))
        })
}