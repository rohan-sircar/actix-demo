use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pets::{PetProfileId, PetProfileUuid};
use crate::types::DbConnection;

/// Delete a pet profile by its UUID
pub fn delete_pet_profile(
    pet_uuid: &PetProfileUuid,
    conn: &mut DbConnection,
) -> Result<(), DomainError> {
    use crate::schema::{
        pet_activities, pet_adoption_details, pet_basic_info,
        pet_location_owner, pet_personality_traits, pet_profile_images,
    };

    let _ = tracing::info!("Deleting pet profile for pet profile UUID {pet_uuid}");

    // First, get the internal ID from the UUID
    let pet_id: PetProfileId = pet_basic_info::table
        .filter(pet_basic_info::uuid.eq(pet_uuid))
        .select(pet_basic_info::id)
        .first(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to get pet profile ID from UUID: {err}"
            ))
        })?;

    conn.transaction::<_, DomainError, _>(|txn| {
        // Delete all related data in reverse order to respect foreign key constraints
        diesel::delete(
            pet_profile_images::table
                .filter(pet_profile_images::pet_profile_uuid.eq(pet_uuid)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete pet profile images: {err}"
            ))
        })?;

        diesel::delete(
            pet_adoption_details::table
                .filter(pet_adoption_details::pet_profile_uuid.eq(pet_uuid)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete adoption details: {err}"
            ))
        })?;

        diesel::delete(
            pet_location_owner::table
                .filter(pet_location_owner::pet_profile_uuid.eq(pet_uuid)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete location/owner info: {err}"
            ))
        })?;

        diesel::delete(
            pet_personality_traits::table
                .filter(pet_personality_traits::pet_profile_uuid.eq(pet_uuid)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete personality traits: {err}"
            ))
        })?;

        diesel::delete(
            pet_activities::table
                .filter(pet_activities::pet_profile_uuid.eq(pet_uuid)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete activities: {err}"
            ))
        })?;

        // Finally, delete the basic pet info using the internal ID
        diesel::delete(pet_basic_info::table.find(pet_id))
            .execute(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to delete pet basic info: {err}"
                ))
            })?;

        Ok(())
    })
}
