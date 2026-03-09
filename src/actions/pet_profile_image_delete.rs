use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pet_profile_images::PetProfileImage;
use crate::models::pets::PetProfileUuid;
use crate::types::DbConnection;

/// Delete a specific pet profile image by its ID
pub fn delete_pet_profile_image(
    pet_uuid: &PetProfileUuid,
    image_id: i32,
    conn: &mut DbConnection,
) -> Result<PetProfileImage, DomainError> {
    use crate::schema::pet_profile_images::dsl::*;

    let _ = tracing::info!(
        "Deleting pet profile image {image_id} for pet profile {pet_uuid}"
    );

    // Find and delete the image, returning the deleted record
    let deleted_image = conn.transaction::<_, DomainError, _>(|txn| {
        diesel::delete(pet_profile_images.find(image_id))
            .filter(pet_profile_uuid.eq(pet_uuid))
            .returning(PetProfileImage::as_returning())
            .get_result(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to delete pet profile image: {err}"
                ))
            })
    })?;

    Ok(deleted_image)
}