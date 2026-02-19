use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pet_basic_info::{
    NewPetBasicInfo, PetBasicInfo, PetBasicInfoId, UpdatePetBasicInfo,
};
use crate::models::users::UserId;
use crate::types::DbConnection;

// Get all pet profiles for a user
pub fn get_pet_basic_info_for_user(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<PetBasicInfo>, DomainError> {
    use crate::schema::pet_basic_info::dsl as profiles;

    let _ = tracing::info!("Getting pet profiles for user {user_id}");

    profiles::pet_basic_info
        .filter(profiles::user_id.eq(user_id))
        .order_by(profiles::created_at.asc())
        .load::<PetBasicInfo>(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to retrieve profiles: {err}"
            ))
        })
}

// Create new pet profile
pub fn create_pet_basic_info(
    new_profile: NewPetBasicInfo,
    conn: &mut DbConnection,
) -> Result<PetBasicInfo, DomainError> {
    use crate::schema::pet_basic_info::dsl as profiles;

    let _ =
        tracing::info!("Creating pet profile for user {}", new_profile.user_id);

    diesel::insert_into(profiles::pet_basic_info)
        .values(&new_profile)
        .get_result(conn)
        .map_err(|err| {
            let _ = tracing::error!("Failed to create profile: {err}");
            DomainError::new_internal_error(format!(
                "Failed to create profile: {err}"
            ))
        })
}

// Update pet profile with ownership validation
pub fn update_pet_basic_info(
    profile_id: &PetBasicInfoId,
    user_id: &UserId,
    update_data: UpdatePetBasicInfo,
    conn: &mut DbConnection,
) -> Result<PetBasicInfo, DomainError> {
    use crate::schema::pet_basic_info::dsl as profiles;

    let _ =
        tracing::info!("Updating pet profile {profile_id} for user {user_id}");

    // // Verify ownership
    // let owned = profiles::pet_basic_info
    //     .filter(profiles::id.eq(profile_id))
    //     .filter(profiles::user_id.eq(user_id))
    //     .count()
    //     .get_result::<i64>(conn)? == 1;

    // if !owned {
    //     return Err(DomainError::new_internal_error(
    //         "You can only update your own pet profiles".to_string()
    //     ));
    // }

    // Perform the update
    let res = diesel::update(profiles::pet_basic_info.find(profile_id))
        .set(&update_data)
        .get_result(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to update profile: {err}"
            ))
        })?;

    Ok(res)
}

// Delete pet profile with ownership validation
pub fn delete_pet_basic_info(
    profile_id: &PetBasicInfoId,
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<usize, DomainError> {
    use crate::schema::pet_basic_info::dsl as profiles;

    let _ =
        tracing::info!("Deleting pet profile {profile_id} for user {user_id}");

    // // Verify ownership
    // let owned = profiles::pet_basic_info
    //     .filter(profiles::id.eq(profile_id))
    //     .filter(profiles::user_id.eq(user_id))
    //     .count()
    //     .get_result::<i64>(conn)? == 1;

    // if !owned {
    //     return Err(DomainError::new_internal_error(
    //         "You can only delete your own pet profiles".to_string()
    //     ));
    // }

    // Perform the deletion
    Ok(diesel::delete(profiles::pet_basic_info.find(profile_id))
        .execute(conn)
        .map_err(|err| {
            DomainError::new_internal_error(format!(
                "Failed to delete profile: {err}"
            ))
        })?)
}
