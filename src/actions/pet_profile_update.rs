use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pet_activities::PetActivities;
use crate::models::pet_adoption_details::PetAdoptionDetails;
use crate::models::pet_basic_info::{PetBasicInfo, PetBasicInfoId};
use crate::models::pet_location_owner::PetLocationOwner;
use crate::models::pet_personality_traits::PetPersonalityTraits;
use crate::models::pet_profile_full::FullPetProfile;
use crate::models::pet_profile_images::PetProfileImage;
use crate::models::pet_profile_update::PetProfileUpdateData;
use crate::types::DbConnection;

/// Update a complete pet profile with all related data in a single transaction
pub fn update_full_pet_profile(
    pet_id: &PetBasicInfoId,
    update_data: PetProfileUpdateData,
    conn: &mut DbConnection,
) -> Result<FullPetProfile, DomainError> {
    use crate::schema::{
        pet_activities, pet_adoption_details, pet_basic_info,
        pet_location_owner, pet_personality_traits, pet_profile_images,
    };

    let _ = tracing::info!("Updating full pet profile for pet ID {pet_id}");

    conn.transaction::<_, DomainError, _>(|txn| {
        // Update basic pet info
        diesel::update(pet_basic_info::table.find(pet_id))
            .set(update_data.to_update_pet_basic_info())
            .execute(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to update pet basic info: {err}"
                ))
            })?;

        // Update personality traits
        diesel::update(pet_personality_traits::table.find(pet_id))
            .set(update_data.to_update_pet_personality_traits())
            .execute(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to update personality traits: {err}"
                ))
            })?;

        // Update activities
        diesel::update(pet_activities::table.find(pet_id))
            .set(update_data.to_update_pet_activities())
            .execute(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to update activities: {err}"
                ))
            })?;

        // Update location/owner info
        diesel::update(pet_location_owner::table.find(pet_id))
            .set(update_data.to_update_pet_location_owner())
            .execute(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to update location/owner info: {err}"
                ))
            })?;

        // Update adoption details
        diesel::update(pet_adoption_details::table.find(pet_id))
            .set(update_data.to_update_pet_adoption_details())
            .execute(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to update adoption details: {err}"
                ))
            })?;

        // Insert new images (not replacing existing ones)
        if !update_data.images.is_empty() {
            diesel::insert_into(pet_profile_images::table)
                .values(update_data.images)
                .execute(txn)
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to insert new pet images: {err}"
                    ))
                })?;
        }

        // Fetch and return the complete updated profile
        let basic_info: PetBasicInfo = pet_basic_info::table
            .find(pet_id)
            .first(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to fetch updated pet basic info: {err}"
                ))
            })?;

        let personality_traits: Option<PetPersonalityTraits> =
            pet_personality_traits::table
                .find(pet_id)
                .first(txn)
                .optional()
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to fetch updated personality traits: {err}"
                    ))
                })?;

        let activities: Option<PetActivities> = pet_activities::table
            .find(pet_id)
            .first(txn)
            .optional()
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to fetch updated activities: {err}"
                ))
            })?;

        let location_owner: Option<PetLocationOwner> =
            pet_location_owner::table
                .find(pet_id)
                .first(txn)
                .optional()
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to fetch updated location/owner info: {err}"
                    ))
                })?;

        let adoption_details: Option<PetAdoptionDetails> =
            pet_adoption_details::table
                .find(pet_id)
                .first(txn)
                .optional()
                .map_err(|err| {
                    DomainError::new_internal_error(format!(
                        "Failed to fetch updated adoption details: {err}"
                    ))
                })?;

        let images: Vec<PetProfileImage> = pet_profile_images::table
            .filter(pet_profile_images::pet_basic_info_id.eq(pet_id))
            .load(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to fetch pet images: {err}"
                ))
            })?;

        Ok(FullPetProfile {
            basic_info,
            personality_traits,
            activities,
            location_owner,
            adoption_details,
            images,
        })
    })
}
