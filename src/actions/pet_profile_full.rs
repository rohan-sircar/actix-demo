use diesel::prelude::*;
use tracing::info;

use crate::errors::DomainError;
use crate::models::pet_profile_full::FullPetProfile;
use crate::models::pet_profile_images::PetProfileImage;
use crate::models::pets::PetActivities;
use crate::models::pets::PetAdoptionDetails;
use crate::models::pets::PetLocationOwner;
use crate::models::pets::PetPersonalityTraits;
use crate::models::pets::{PetBasicInfo, PetProfileId};
use crate::models::users::UserId;
use crate::types::DbConnection;

/// Helper function to fetch pet images
fn fetch_pet_images(
    pet_id: &PetProfileId,
    txn: &mut DbConnection,
) -> Result<Vec<PetProfileImage>, DomainError> {
    use crate::schema::pet_profile_images::dsl as images;

    images::pet_profile_images
        .filter(images::pet_profile_id.eq(pet_id))
        .order_by(images::sort_order.asc())
        .select(PetProfileImage::as_select())
        .load::<PetProfileImage>(txn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to retrieve pet images: {err}"
            ))
        })
}

/// Helper function to fetch all related data for a single pet
fn fetch_pet_related_data(
    pet_id: &PetProfileId,
    txn: &mut DbConnection,
) -> Result<
    (
        Option<PetPersonalityTraits>,
        Option<PetActivities>,
        Option<PetLocationOwner>,
        Option<PetAdoptionDetails>,
        Vec<PetProfileImage>,
    ),
    DomainError,
> {
    use crate::schema::pet_activities::dsl as activities;
    use crate::schema::pet_adoption_details::dsl as adoption_details;
    use crate::schema::pet_location_owner::dsl as location_owner;
    use crate::schema::pet_personality_traits::dsl as personality_traits;

    // Fetch personality traits
    let _ = info!("Fetching personality traits for pet {pet_id}");
    let personality_traits = personality_traits::pet_personality_traits
        .filter(personality_traits::pet_profile_id.eq(pet_id))
        .select(PetPersonalityTraits::as_select())
        .first::<PetPersonalityTraits>(txn)
        .optional()?;
    let _ = tracing::debug!(
        "Personality traits result for pet {pet_id}: {:?}",
        &personality_traits
    );

    // Fetch activities
    let _ = info!("Fetching activities for pet {pet_id}");
    let activities = activities::pet_activities
        .filter(activities::pet_profile_id.eq(pet_id))
        .select(PetActivities::as_select())
        .first::<PetActivities>(txn)
        .optional()?;
    let _ = tracing::debug!(
        "Activities result for pet {pet_id}: {:?}",
        &activities
    );

    // Fetch location/owner info
    let _ = info!("Fetching location/owner info for pet {pet_id}");
    let location_owner = location_owner::pet_location_owner
        .filter(location_owner::pet_profile_id.eq(pet_id))
        .first::<PetLocationOwner>(txn)
        .optional()?;
    let _ = tracing::debug!(
        "Location/owner info result for pet {pet_id}: {:?}",
        &location_owner
    );

    // Fetch adoption details
    let _ = info!("Fetching adoption details for pet {pet_id}");
    let adoption_details = adoption_details::pet_adoption_details
        .filter(adoption_details::pet_profile_id.eq(pet_id))
        .select(PetAdoptionDetails::as_select())
        .first::<PetAdoptionDetails>(txn)
        .optional()?;
    let _ = tracing::debug!(
        "Adoption details result for pet {pet_id}: {:?}",
        &adoption_details
    );

    // Fetch images
    let _ = info!("Fetching images for pet {pet_id}");
    let images = fetch_pet_images(pet_id, txn)?;
    let _ =
        tracing::debug!("Images result for pet {pet_id}: {:?}", &images.len());

    Ok((
        personality_traits,
        activities,
        location_owner,
        adoption_details,
        images,
    ))
}

pub fn check_pet_profile_exists(
    pet_id: &PetProfileId,
    conn: &mut DbConnection,
) -> Result<bool, DomainError> {
    use crate::schema::pet_basic_info::dsl as basic_info;

    let _ = tracing::info!("Getting complete pet profile for pet {pet_id}");

    // Fetch basic info
    let res = basic_info::pet_basic_info
        .find(pet_id)
        .select(basic_info::id)
        .first::<PetProfileId>(conn)
        .optional()
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to retrieve pet basic info: {err}"
            ))
        })?;

    Ok(res.is_some())
}

// Get complete pet profile with all related data for a specific pet
pub fn get_full_pet_profile(
    pet_id: &PetProfileId,
    conn: &mut DbConnection,
) -> Result<Option<FullPetProfile>, DomainError> {
    use crate::schema::pet_basic_info::dsl as basic_info;

    let _ = tracing::info!("Getting complete pet profile for pet {pet_id}");

    // Execute all database operations within a single transaction
    conn.transaction::<_, DomainError, _>(|txn| {
        // Fetch basic info
        let res = basic_info::pet_basic_info
            .find(pet_id)
            .first::<PetBasicInfo>(txn)
            .optional()
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to retrieve pet basic info: {err}"
                ))
            })?;

        if let Some(pet_basic_info) = res {
            let (
                personality_traits,
                activities,
                location_owner,
                adoption_details,
                images,
            ) = fetch_pet_related_data(pet_id, txn)?;

            // Fetch all related data using helper function
            Ok(Some(FullPetProfile {
                basic_info: pet_basic_info,
                personality_traits,
                activities,
                location_owner,
                adoption_details,
                images,
            }))
        } else {
            Ok(None)
        }
    })
}

// Get all pet profiles for a user with complete data
pub fn get_full_pet_profiles_for_user(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<FullPetProfile>, DomainError> {
    use crate::schema::pet_basic_info::dsl as basic_info;

    let _ = tracing::info!("Getting complete pet profiles for user {user_id}");

    // Execute all database operations within a single transaction
    conn.transaction::<_, DomainError, _>(|txn| {
        // Get all basic info for user
        let pet_basic_infos: Vec<PetBasicInfo> = basic_info::pet_basic_info
            .filter(basic_info::user_id.eq(user_id))
            .order_by(basic_info::created_at.asc())
            .load::<PetBasicInfo>(txn)
            .map_err(|err| {
                DomainError::new_internal_error(format!(
                    "Failed to retrieve pet basic info: {err}"
                ))
            })?;

        // Collect all profiles using helper functions
        let profiles = pet_basic_infos
            .into_iter()
            .map(|basic_info| {
                let (
                    personality_traits,
                    activities,
                    location_owner,
                    adoption_details,
                    images,
                ) = fetch_pet_related_data(&basic_info.id, txn)?;

                Ok(FullPetProfile {
                    basic_info,
                    personality_traits,
                    activities,
                    location_owner,
                    adoption_details,
                    images,
                })
            })
            .collect::<Result<Vec<FullPetProfile>, DomainError>>()?;

        Ok(profiles)
    })
}
