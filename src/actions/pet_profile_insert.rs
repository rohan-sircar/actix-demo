use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pet_profile_full::FullPetProfile;
use crate::models::pet_profile_images::{NewPetProfileImage, PetProfileImage};
use crate::models::pet_profile_insert::PetProfileInsertData;
use crate::models::pets::{NewPetActivities, PetActivities};
use crate::models::pets::{NewPetAdoptionDetails, PetAdoptionDetails};
use crate::models::pets::{NewPetBasicInfo, PetBasicInfo};
use crate::models::pets::{NewPetLocationOwner, PetLocationOwner};
use crate::models::pets::{NewPetPersonalityTraits, PetPersonalityTraits};
use crate::types::DbConnection;

/// Insert a complete pet profile with all related data in a single transaction
pub fn create_full_pet_profile(
    pet_data: PetBasicInfo,
    personality_data: NewPetPersonalityTraits,
    activities_data: NewPetActivities,
    location_data: NewPetLocationOwner,
    adoption_data: NewPetAdoptionDetails,
    images: Vec<NewPetProfileImage>,
    conn: &mut DbConnection,
) -> Result<FullPetProfile, DomainError> {
    use crate::schema::{
        pet_activities, pet_adoption_details, pet_location_owner,
        pet_personality_traits, pet_profile_images,
    };

    let user_id = &pet_data.user_id;
    let _ = tracing::info!("Creating full pet profile for user {user_id}");

    // Insert personality traits
    let personality_traits: PetPersonalityTraits =
        diesel::insert_into(pet_personality_traits::table)
            .values(personality_data)
            .get_result(conn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to insert personality traits: {err}"
                ))
            })?;

    // Insert activities
    let activities: PetActivities = diesel::insert_into(pet_activities::table)
        .values(activities_data)
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to insert activities: {err}"
            ))
        })?;

    // Insert location/owner info
    let location_owner: PetLocationOwner =
        diesel::insert_into(pet_location_owner::table)
            .values(location_data)
            .get_result(conn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to insert location/owner info: {err}"
                ))
            })?;

    // Insert adoption details
    let adoption_details: PetAdoptionDetails =
        diesel::insert_into(pet_adoption_details::table)
            .values(adoption_data)
            .get_result(conn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to insert adoption details: {err}"
                ))
            })?;

    // Insert images
    let inserted_images: Vec<PetProfileImage> = if images.is_empty() {
        Vec::new()
    } else {
        diesel::insert_into(pet_profile_images::table)
            .values(images)
            .get_results(conn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to insert pet images: {err}"
                ))
            })?
    };

    // Return the complete profile using your FullPetProfile struct
    Ok(FullPetProfile {
        basic_info: pet_data,
        personality_traits: Some(personality_traits),
        activities: Some(activities),
        location_owner: Some(location_owner),
        adoption_details: Some(adoption_details),
        images: inserted_images,
    })
}

/// Insert a complete pet profile using the unified insert data struct
pub fn create_pet_profile_from_insert_data(
    insert_data: PetProfileInsertData,
    conn: &mut DbConnection,
) -> Result<FullPetProfile, DomainError> {
    use crate::actions::pet_profile_insert::create_full_pet_profile;

    // Convert to individual New structs
    let pet_data = insert_data.to_new_pet_basic_info()?;
    let _ = tracing::debug!(
        "Converted insert data to pet basic info: {:?}",
        pet_data
    );

    conn.transaction::<_, DomainError, _>(|txn| {
        // We need to insert basic info first to get the pet ID
        let pet_data =
            crate::actions::pet_profile_insert::create_pet_basic_info(
                pet_data.clone(),
                txn,
            )?;
        let pet_id = &pet_data.id;
        let _ = tracing::info!("Created pet basic info with ID: {}", pet_id);

        let personality_data =
            insert_data.to_new_pet_personality_traits(&pet_id);
        let _ = tracing::debug!(
            "Converted insert data to personality traits: {:?}",
            personality_data
        );

        let activities_data = insert_data.to_new_pet_activities(&pet_id);
        let _ = tracing::debug!(
            "Converted insert data to activities: {:?}",
            activities_data
        );

        let location_data = insert_data.to_new_pet_location_owner(&pet_id);
        let _ = tracing::debug!(
            "Converted insert data to location/owner info: {:?}",
            location_data
        );

        let adoption_data = insert_data.to_new_pet_adoption_details(&pet_id);
        let _ = tracing::debug!(
            "Converted insert data to adoption details: {:?}",
            adoption_data
        );

        let images = insert_data.to_new_pet_profile_images(&pet_id);
        let _ =
            tracing::debug!("Converted insert data to images: {:?}", images);

        // Use the existing create_full_pet_profile function
        create_full_pet_profile(
            pet_data,
            personality_data,
            activities_data,
            location_data,
            adoption_data,
            images,
            txn,
        )
    })
}

/// Insert basic pet information
pub fn create_pet_basic_info(
    pet_data: NewPetBasicInfo,
    conn: &mut DbConnection,
) -> Result<PetBasicInfo, DomainError> {
    use crate::schema::pet_basic_info;

    let result = diesel::insert_into(pet_basic_info::table)
        .values(&pet_data)
        .returning(PetBasicInfo::as_returning())
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to create pet basic info: {err}"
            ))
        })?;

    Ok(result)
}

/// Insert pet personality traits
pub fn create_pet_personality_traits(
    personality_data: NewPetPersonalityTraits,
    conn: &mut DbConnection,
) -> Result<PetPersonalityTraits, DomainError> {
    use crate::schema::pet_personality_traits;

    diesel::insert_into(pet_personality_traits::table)
        .values(&personality_data)
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to create personality traits: {err}"
            ))
        })
}

/// Insert pet activities
pub fn create_pet_activities(
    activities_data: NewPetActivities,
    conn: &mut DbConnection,
) -> Result<PetActivities, DomainError> {
    use crate::schema::pet_activities;

    diesel::insert_into(pet_activities::table)
        .values(&activities_data)
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to create pet activities: {err}"
            ))
        })
}

/// Insert pet location and owner information
pub fn create_pet_location_owner(
    location_data: NewPetLocationOwner,
    conn: &mut DbConnection,
) -> Result<PetLocationOwner, DomainError> {
    use crate::schema::pet_location_owner;

    diesel::insert_into(pet_location_owner::table)
        .values(&location_data)
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to create pet location/owner info: {err}"
            ))
        })
}

/// Insert pet adoption details
pub fn create_pet_adoption_details(
    adoption_data: NewPetAdoptionDetails,
    conn: &mut DbConnection,
) -> Result<PetAdoptionDetails, DomainError> {
    use crate::schema::pet_adoption_details;

    diesel::insert_into(pet_adoption_details::table)
        .values(&adoption_data)
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to create pet adoption details: {err}"
            ))
        })
}
