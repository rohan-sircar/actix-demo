use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pet_profile_full::FullPetProfile;
use crate::models::pet_profile_images::PetProfileImage;
use crate::models::pet_profile_update::PetProfileUpdateData;
use crate::models::pets::PetActivities;
use crate::models::pets::PetAdoptionDetails;
use crate::models::pets::PetLocationOwner;
use crate::models::pets::PetPersonalityTraits;
use crate::models::pets::{PetBasicInfo, PetProfileId};
use crate::types::DbConnection;

/// Update a complete pet profile with all related data in a single transaction
pub fn update_full_pet_profile(
    pet_id: &PetProfileId,
    update_data: PetProfileUpdateData,
    conn: &mut DbConnection,
) -> Result<FullPetProfile, DomainError> {
    use crate::schema::{
        pet_activities, pet_adoption_details, pet_basic_info,
        pet_location_owner, pet_personality_traits, pet_profile_images,
    };

    let _ = tracing::info!("Updating full pet profile for pet ID {pet_id}");

    conn.transaction::<_, DomainError, _>(|txn| {
        // Update basic pet info only if data is provided
        if update_data.basic_info.is_some() {
            let basic_info = update_data.to_update_pet_basic_info()?;
            if let Some(basic_info) = basic_info {
                diesel::update(pet_basic_info::table.find(pet_id))
                    .set(basic_info)
                    .execute(txn)
                    .map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to update pet basic info: {err}"
                        ))
                    })?;
            }
        }

        // Update personality traits only if data is provided
        if update_data.personality_traits.is_some() {
            let personality_traits =
                update_data.to_update_pet_personality_traits();
            if let Some(personality_traits) = personality_traits {
                diesel::update(pet_personality_traits::table.find(pet_id))
                    .set(personality_traits)
                    .execute(txn)
                    .map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to update personality traits: {err}"
                        ))
                    })?;
            }
        }

        // Update activities only if data is provided
        if update_data.activities.is_some() {
            let activities = update_data.to_update_pet_activities();
            if let Some(activities) = activities {
                diesel::update(pet_activities::table.find(pet_id))
                    .set(activities)
                    .execute(txn)
                    .map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to update activities: {err}"
                        ))
                    })?;
            }
        }

        // Update location/owner info only if data is provided
        if update_data.location_owner.is_some() {
            let location_owner = update_data.to_update_pet_location_owner();
            if let Some(location_owner) = location_owner {
                diesel::update(pet_location_owner::table.find(pet_id))
                    .set(location_owner)
                    .execute(txn)
                    .map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to update location/owner info: {err}"
                        ))
                    })?;
            }
        }

        // Update adoption details only if data is provided
        if update_data.adoption_details.is_some() {
            let adoption_details = update_data.to_update_pet_adoption_details();
            if let Some(adoption_details) = adoption_details {
                diesel::update(pet_adoption_details::table.find(pet_id))
                    .set(adoption_details)
                    .execute(txn)
                    .map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to update adoption details: {err}"
                        ))
                    })?;
            }
        }

        // Insert new images (not replacing existing ones) only if data is provided
        if let Some(images) = update_data.images {
            if !images.is_empty() {
                diesel::insert_into(pet_profile_images::table)
                    .values(images)
                    .execute(txn)
                    .map_err(|err| {
                        DomainError::new_internal_error(format!(
                            "Failed to insert new pet images: {err}"
                        ))
                    })?;
            }
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
            .filter(pet_profile_images::pet_profile_id.eq(pet_id))
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
