use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::likes::{
    CreateLike, Like, LikeDirection, LikeResponse, LikeWithPet, NewLike,
};
use crate::models::pets::{PetId, PetUuid};
use crate::models::users::UserId;
use crate::types::DbConnection;

/// Creates a like or dislike for a pet. Detects mutual matches.
pub fn create_like(
    user_id: &UserId,
    create: CreateLike,
    conn: &mut DbConnection,
) -> Result<LikeResponse, DomainError> {
    use crate::schema::likes::dsl as likes;
    use crate::schema::pets::dsl as pets;

    let CreateLike {
        pet_uuid,
        direction,
    } = create;

    conn.transaction::<_, DomainError, _>(|conn| {
        // Find the pet and its owner
        let (pet_id, pet_owner_id): (PetId, UserId) = pets::pets
            .select((pets::id, pets::user_id))
            .filter(pets::pet_uuid.eq(&pet_uuid))
            .first(conn)?;

        // Prevent liking your own pet
        if pet_owner_id == *user_id {
            return Err(DomainError::new_conflict_error(
                "You cannot like your own pet".to_string(),
            ));
        }

        // Check if the user already liked/disliked this pet
        let existing: Result<Like, _> = likes::likes
            .filter(likes::user_id.eq(*user_id).and(likes::pet_id.eq(pet_id)))
            .first(conn);

        if existing.is_ok() {
            return Err(DomainError::new_conflict_error(
                "You have already interacted with this pet".to_string(),
            ));
        }

        // Insert the new like
        let new_like =
            NewLike::new(*user_id, pet_owner_id, pet_id, direction.clone());
        let inserted_id: i32 = diesel::insert_into(likes::likes)
            .values(&new_like)
            .returning(likes::id)
            .get_result(conn)?;

        // Check for mutual match (only for "like" direction)
        let is_match = if direction == LikeDirection::Like {
            use crate::schema::pets::dsl::{
                id as pet_id_col, pets, user_id as owner_col,
            };

            let reciprocal: Result<Like, _> = likes::likes
                .filter(
                    likes::user_id
                        .eq(pet_owner_id)
                        .and(
                            likes::pet_id.eq_any(
                                pets.select(pet_id_col)
                                    .filter(owner_col.eq(*user_id)),
                            ),
                        )
                        .and(likes::direction.eq(LikeDirection::Like)),
                )
                .first(conn);

            reciprocal.is_ok()
        } else {
            false
        };

        // If it's a match, update the current like record
        if is_match {
            diesel::update(likes::likes.filter(likes::id.eq(inserted_id)))
                .set((
                    likes::is_match.eq(true),
                    likes::matched_at.eq(diesel::dsl::now),
                ))
                .execute(conn)?;
        }

        // Fetch the inserted like
        let like: Like =
            likes::likes.filter(likes::id.eq(inserted_id)).first(conn)?;

        Ok(LikeResponse::from((&like, &pet_uuid)))
    })
}

/// Returns likes the user has sent (pets the user liked/disliked).
pub fn list_likes_sent(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<LikeWithPet>, DomainError> {
    use crate::schema::likes::dsl as likes;

    let like_rows: Vec<Like> = likes::likes
        .filter(
            likes::user_id
                .eq(*user_id)
                .and(likes::direction.eq(LikeDirection::Like)),
        )
        .order(likes::matched_at.desc())
        .load(conn)?;

    like_rows
        .into_iter()
        .map(|like| {
            let pet = fetch_pet_with_image(&like.pet_id, conn);
            match pet {
                Ok((pet_uuid, pet_name, species, image_uuid)) => {
                    Ok(LikeWithPet {
                        pet_uuid,
                        pet_name,
                        species,
                        primary_image_uuid: image_uuid.map(|u| u.to_string()),
                        is_match: like.is_match,
                        matched_at: like.matched_at,
                        created_at: like.created_at,
                        liker: None,
                    })
                }
                Err(_) => Err(DomainError::new_internal_error(format!(
                    "Pet not found for like id {}",
                    like.id
                ))),
            }
        })
        .collect()
}

/// Returns likes the user has received (other users who liked this user's pets).
pub fn list_likes_received(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<LikeWithPet>, DomainError> {
    use crate::schema::likes::dsl as likes;

    let like_rows: Vec<Like> = likes::likes
        .filter(
            likes::pet_owner_id
                .eq(*user_id)
                .and(likes::direction.eq(LikeDirection::Like)),
        )
        .order(likes::created_at.desc())
        .load(conn)?;

    like_rows
        .into_iter()
        .map(|like| {
            let pet = fetch_pet_with_image(&like.pet_id, conn);
            match pet {
                Ok((pet_uuid, pet_name, species, image_uuid)) => {
                    let liker = fetch_liker_info(&like.user_id, conn).ok();
                    Ok(LikeWithPet {
                        pet_uuid,
                        pet_name,
                        species,
                        primary_image_uuid: image_uuid.map(|u| u.to_string()),
                        is_match: like.is_match,
                        matched_at: like.matched_at,
                        created_at: like.created_at,
                        liker,
                    })
                }
                Err(_) => Err(DomainError::new_internal_error(format!(
                    "Pet not found for like id {}",
                    like.id
                ))),
            }
        })
        .collect()
}

/// Fetches the liker's user info (user_uuid, display_name, avatar_url, pets_owned).
fn fetch_liker_info(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<crate::models::pets::PublicPetOwner, DomainError> {
    use crate::schema::pets::dsl as pets;
    use crate::schema::profiles::dsl as profiles;
    use crate::schema::users::dsl as users;

    let (user_uuid_opt, display_name, avatar_url) = users::users
        .inner_join(profiles::profiles)
        .filter(users::id.eq(*user_id))
        .select((
            users::user_uuid,
            profiles::display_name,
            profiles::avatar_url,
        ))
        .first::<(uuid::Uuid, Option<String>, Option<String>)>(conn)
        .map_err(DomainError::from)?;

    let pets_owned: i64 = pets::pets
        .filter(pets::user_id.eq(*user_id))
        .count()
        .get_result(conn)
        .map_err(DomainError::from)?;

    Ok(crate::models::pets::PublicPetOwner {
        user_uuid: crate::models::users::UserUuid::try_from(
            user_uuid_opt.to_string(),
        )
        .map_err(|e: String| {
            DomainError::new_internal_error(format!("Invalid user UUID: {}", e))
        })?,
        display_name,
        avatar_url,
        pets_owned: pets_owned as u32,
    })
}

/// Checks if a user has already interacted with a specific pet (liked/disliked).
/// Returns Some(direction, is_match) if interaction exists, None otherwise.
pub fn get_pet_interaction(
    user_id: &UserId,
    pet_uuid: &PetUuid,
    conn: &mut DbConnection,
) -> Result<Option<(LikeDirection, bool)>, DomainError> {
    use crate::schema::likes::dsl as likes;
    use crate::schema::pets::dsl as pets;

    let pet_id: PetId = pets::pets
        .select(pets::id)
        .filter(pets::pet_uuid.eq(pet_uuid))
        .first(conn)?;

    let interaction: Option<Like> = likes::likes
        .filter(likes::user_id.eq(*user_id).and(likes::pet_id.eq(pet_id)))
        .first(conn)
        .optional()?;

    Ok(interaction.map(|like| (like.direction, like.is_match)))
}

/// Returns mutual matches with both pets involved for the current user.
pub fn list_matches_with_pets(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<crate::models::likes::MatchWithPets>, DomainError> {
    use crate::schema::likes::dsl as likes;

    let like_rows: Vec<Like> = likes::likes
        .filter(
            likes::user_id
                .eq(*user_id)
                .and(likes::direction.eq(LikeDirection::Like))
                .and(likes::is_match.eq(true)),
        )
        .order(likes::matched_at.desc())
        .load(conn)?;

    like_rows
        .into_iter()
        .map(|like| {
            let other_user_id = like.pet_owner_id;
            let their_pet = fetch_pet_with_image(&like.pet_id, conn)?;
            let their_pet_info = crate::models::likes::MatchPetInfo {
                pet_uuid: their_pet.0,
                pet_name: their_pet.1,
                species: their_pet.2,
                primary_image_uuid: their_pet.3.map(|u| u.to_string()),
            };

            // Find the reciprocal like (other user liked one of current user's pets)
            use crate::schema::pets::dsl as pets;
            let my_pet_id: PetId = likes::likes
                .inner_join(pets::pets)
                .filter(
                    likes::user_id
                        .eq(other_user_id)
                        .and(pets::user_id.eq(*user_id))
                        .and(likes::direction.eq(LikeDirection::Like)),
                )
                .select(pets::id)
                .first(conn)?;

            let my_pet = fetch_pet_with_image(&my_pet_id, conn)?;
            let my_pet_info = crate::models::likes::MatchPetInfo {
                pet_uuid: my_pet.0,
                pet_name: my_pet.1,
                species: my_pet.2,
                primary_image_uuid: my_pet.3.map(|u| u.to_string()),
            };

            let other_owner = fetch_liker_info(&other_user_id, conn)?;

            Ok(crate::models::likes::MatchWithPets {
                liked_pet: their_pet_info,
                liked_by_other_pet: my_pet_info,
                other_owner_name: other_owner.display_name,
                other_owner_avatar_url: other_owner.avatar_url,
                other_user_uuid: other_owner.user_uuid,
                is_match: like.is_match,
                matched_at: like.matched_at,
            })
        })
        .collect()
}

fn fetch_pet_with_image(
    pet_id: &PetId,
    conn: &mut DbConnection,
) -> Result<(PetUuid, String, String, Option<uuid::Uuid>), DomainError> {
    use crate::schema::pets::dsl as pets;

    let (pet_uuid_raw, pet_name, species): (uuid::Uuid, String, String) =
        pets::pets
            .select((pets::pet_uuid, pets::name, pets::species))
            .filter(pets::id.eq(*pet_id))
            .first(conn)
            .map_err(DomainError::from)?;

    let pet_uuid: PetUuid = PetUuid::try_from(pet_uuid_raw.to_string())
        .map_err(|e: String| {
            DomainError::new_internal_error(format!("Invalid pet UUID: {}", e))
        })?;

    let image_uuid: Option<uuid::Uuid> = {
        use crate::schema::pet_images::dsl as pet_images;
        let result: Result<uuid::Uuid, _> = pet_images::pet_images
            .select(pet_images::uuid)
            .filter(
                pet_images::pet_id
                    .eq(*pet_id)
                    .and(pet_images::is_primary.eq(true)),
            )
            .first(conn);
        result.ok()
    };

    Ok((pet_uuid, pet_name, species, image_uuid))
}
