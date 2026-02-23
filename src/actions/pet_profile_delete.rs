use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pets::PetProfileId;
use crate::types::DbConnection;

/// Delete a pet profile by its ID
pub fn delete_pet_profile(
    pet_id: &PetProfileId,
    conn: &mut DbConnection,
) -> Result<(), DomainError> {
    use crate::schema::{
        pet_activities, pet_adoption_details, pet_basic_info,
        pet_location_owner, pet_personality_traits, pet_profile_images,
    };

    let _ = tracing::info!("Deleting pet profile for pet profile ID {pet_id}");

    conn.transaction::<_, DomainError, _>(|txn| {
        // Delete all related data in reverse order to respect foreign key constraints
        diesel::delete(
            pet_profile_images::table
                .filter(pet_profile_images::pet_profile_id.eq(pet_id)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete pet profile images: {err}"
            ))
        })?;

        diesel::delete(
            pet_adoption_details::table
                .filter(pet_adoption_details::pet_profile_id.eq(pet_id)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete adoption details: {err}"
            ))
        })?;

        diesel::delete(
            pet_location_owner::table
                .filter(pet_location_owner::pet_profile_id.eq(pet_id)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete location/owner info: {err}"
            ))
        })?;

        diesel::delete(
            pet_personality_traits::table
                .filter(pet_personality_traits::pet_profile_id.eq(pet_id)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete personality traits: {err}"
            ))
        })?;

        diesel::delete(
            pet_activities::table
                .filter(pet_activities::pet_profile_id.eq(pet_id)),
        )
        .execute(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete activities: {err}"
            ))
        })?;

        // Finally, delete the basic pet info
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
