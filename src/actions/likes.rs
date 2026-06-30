use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::likes::{
    CreateLike, Like, LikeDirection, LikeId, LikeResponse, LikeWithPet,
    MatchPetInfo, NewLike, NewMatch, PetInteractionResponse,
};
use crate::models::pets::{PetId, PetImageUuid, PetName, PetSpecies, PetUuid};
use crate::models::users::UserId;
use crate::types::DbConnection;

/// Creates a like or dislike for a pet. Creates a match if an explicit reciprocal_pet_uuid is provided.
pub fn create_like(
    user_id: &UserId,
    create: CreateLike,
    conn: &mut DbConnection,
) -> Result<LikeResponse, DomainError> {
    use crate::schema::likes::dsl as likes;
    use crate::schema::matches::dsl as matches;
    use crate::schema::pets::dsl as pets;

    let CreateLike {
        pet_uuid,
        direction,
        reciprocal_pet_uuid,
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
        let inserted_id: LikeId = diesel::insert_into(likes::likes)
            .values(&new_like)
            .returning(likes::id)
            .get_result(conn)?;

        // Create match if an explicit reciprocal_pet_uuid was provided
        if let Some(recip_pet_uuid) = reciprocal_pet_uuid {
            use crate::schema::pets::dsl::{
                id as recip_pet_id_col, pet_uuid, pets as recip_pets,
                user_id as recip_user_id,
            };

            // Verify the reciprocal pet belongs to the current user
            let recip_pet_id: PetId = recip_pets
                .select(recip_pet_id_col)
                .filter(
                    pet_uuid
                        .eq(&recip_pet_uuid)
                        .and(recip_user_id.eq(*user_id)),
                )
                .first(conn)
                .map_err(|_| {
                    DomainError::new_bad_input_error(
                        "Reciprocal pet not found or does not belong to you"
                            .to_string(),
                    )
                })?;

            // Check if reciprocal like already exists
            let reciprocal: Like = likes::likes
                .filter(
                    likes::pet_id
                        .eq(recip_pet_id)
                        .and(likes::user_id.eq(pet_owner_id))
                        .and(likes::direction.eq(LikeDirection::Like)),
                )
                .first(conn)
                .map_err(|_| {
                    DomainError::new_bad_input_error(
                        "No matching reciprocal like found".to_string(),
                    )
                })?;

            // Determine which like_id is smaller for the constraint like_id_a < like_id_b
            let (a, b) = if reciprocal.id < inserted_id {
                (reciprocal.id, inserted_id)
            } else {
                (inserted_id, reciprocal.id)
            };

            // Insert into matches table
            let new_match = NewMatch::new(a, b);
            diesel::insert_into(matches::matches)
                .values(&new_match)
                .execute(conn)?;
        } else {
            // Auto-detect reciprocal likes: check if pet_owner has liked any of user_id's pets
            use crate::schema::pets::dsl::{
                id as my_pet_id_col, pets as my_pets, user_id as my_owner_id,
            };

            let my_pet_ids: Vec<PetId> = my_pets
                .select(my_pet_id_col)
                .filter(my_owner_id.eq(*user_id))
                .load(conn)?;

            if !my_pet_ids.is_empty() {
                use crate::schema::likes::dsl as likes_dsl;

                let reciprocal_like: Option<Like> = likes_dsl::likes
                    .filter(
                        likes_dsl::pet_id
                            .eq_any(&my_pet_ids)
                            .and(likes_dsl::user_id.eq(pet_owner_id))
                            .and(likes_dsl::direction.eq(LikeDirection::Like)),
                    )
                    .first(conn)
                    .optional()?;

                if let Some(reciprocal) = reciprocal_like {
                    let (a, b) = if reciprocal.id < inserted_id {
                        (reciprocal.id, inserted_id)
                    } else {
                        (inserted_id, reciprocal.id)
                    };

                    let new_match = NewMatch::new(a, b);
                    let _ = diesel::insert_into(matches::matches)
                        .values(&new_match)
                        .execute(conn);
                }
            }
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
        .order(likes::created_at.desc())
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
        .first::<(
            uuid::Uuid,
            Option<crate::models::users::DisplayName>,
            Option<String>,
        )>(conn)
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
/// Returns interaction status and potential reciprocal match pets.
pub fn get_pet_interaction(
    user_id: &UserId,
    pet_uuid: &PetUuid,
    conn: &mut DbConnection,
) -> Result<PetInteractionResponse, DomainError> {
    use crate::schema::likes::dsl as likes;
    use crate::schema::pets::dsl as pets;

    let (pet_id, pet_owner_id): (PetId, UserId) = pets::pets
        .select((pets::id, pets::user_id))
        .filter(pets::pet_uuid.eq(pet_uuid))
        .first(conn)?;

    let interaction: Option<Like> = likes::likes
        .filter(likes::user_id.eq(*user_id).and(likes::pet_id.eq(pet_id)))
        .first(conn)
        .optional()?;

    // Find potential reciprocal matches: pets owned by current user that pet_owner has liked
    let potential_matches: Vec<MatchPetInfo> = {
        use crate::schema::pets::dsl::{
            id as my_pet_id_col, pets as my_pets, user_id as my_owner_id_col,
        };

        let recip_like_rows: Vec<Like> = likes::likes
            .filter(
                likes::user_id
                    .eq(pet_owner_id)
                    .and(
                        likes::pet_id.eq_any(
                            my_pets
                                .select(my_pet_id_col)
                                .filter(my_owner_id_col.eq(*user_id)),
                        ),
                    )
                    .and(likes::direction.eq(LikeDirection::Like)),
            )
            .load(conn)?;

        recip_like_rows
            .into_iter()
            .filter_map(|like| match fetch_pet_with_image(&like.pet_id, conn) {
                Ok((p_uuid, p_name, p_species, p_image)) => {
                    Some(MatchPetInfo {
                        pet_uuid: p_uuid,
                        pet_name: p_name,
                        species: p_species,
                        primary_image_uuid: p_image.map(|u| u.to_string()),
                    })
                }
                Err(_) => None,
            })
            .collect()
    };

    Ok(PetInteractionResponse {
        interacted: interaction.is_some(),
        direction: interaction.map(|like| like.direction),
        potential_matches,
    })
}

/// Returns mutual matches with both pets involved for the current user.
pub fn list_matches_with_pets(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<crate::models::likes::MatchWithPets>, DomainError> {
    use crate::schema::likes::dsl as likes;

    // Get the user's like IDs to filter matches at the SQL level
    let user_like_ids: Vec<crate::models::likes::LikeId> = likes::likes
        .filter(likes::user_id.eq(*user_id))
        .select(likes::id)
        .load(conn)?;

    // Short-circuit if user has no likes
    if user_like_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Load only matches involving this user via SQL IN filter
    use crate::models::likes::{Like, LikeId, Match};
    use crate::schema::matches::dsl as matches_dsl;

    let user_matches: Vec<Match> = matches_dsl::matches
        .filter(
            matches_dsl::like_id_a
                .eq_any(&user_like_ids)
                .or(matches_dsl::like_id_b.eq_any(&user_like_ids)),
        )
        .order(matches_dsl::matched_at.desc())
        .load(conn)?;

    // Batch-load all likes involved in the user's matches
    let like_ids: Vec<LikeId> = user_matches
        .iter()
        .flat_map(|m| [m.like_id_a, m.like_id_b])
        .collect();
    let likes_map: std::collections::HashMap<LikeId, Like> =
        if like_ids.is_empty() {
            std::collections::HashMap::new()
        } else {
            likes::likes
                .filter(likes::id.eq_any(&like_ids))
                .load(conn)?
                .into_iter()
                .map(|l: Like| (l.id, l))
                .collect()
        };

    let mut result = Vec::new();

    for match_row in &user_matches {
        let like_a = match likes_map.get(&match_row.like_id_a) {
            Some(l) => l.clone(),
            None => continue,
        };
        let like_b = match likes_map.get(&match_row.like_id_b) {
            Some(l) => l.clone(),
            None => continue,
        };

        let (current_like, other_like) = match (&like_a, &like_b) {
            (a, b) if a.user_id == *user_id => (a, b),
            (a, b) if b.user_id == *user_id => (b, a),
            _ => continue,
        };

        let other_user_id = other_like.pet_owner_id;

        let their_pet = match fetch_pet_with_image(&other_like.pet_id, conn) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let their_pet_info = crate::models::likes::MatchPetInfo {
            pet_uuid: their_pet.0,
            pet_name: their_pet.1,
            species: their_pet.2,
            primary_image_uuid: their_pet.3.map(|u| u.to_string()),
        };

        let my_pet = match fetch_pet_with_image(&current_like.pet_id, conn) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let my_pet_info = crate::models::likes::MatchPetInfo {
            pet_uuid: my_pet.0,
            pet_name: my_pet.1,
            species: my_pet.2,
            primary_image_uuid: my_pet.3.map(|u| u.to_string()),
        };

        let other_owner = match fetch_liker_info(&other_user_id, conn) {
            Ok(o) => o,
            Err(_) => continue,
        };

        result.push(crate::models::likes::MatchWithPets {
            liked_pet: their_pet_info,
            liked_by_other_pet: my_pet_info,
            other_owner_name: other_owner.display_name,
            other_owner_avatar_url: other_owner.avatar_url,
            other_user_uuid: other_owner.user_uuid,
            matched_at: Some(match_row.matched_at),
        });
    }

    Ok(result)
}

fn fetch_pet_with_image(
    pet_id: &PetId,
    conn: &mut DbConnection,
) -> Result<(PetUuid, PetName, PetSpecies, Option<PetImageUuid>), DomainError> {
    use crate::schema::pets::dsl as pets;

    let (pet_uuid, pet_name, species): (PetUuid, PetName, PetSpecies) =
        pets::pets
            .select((pets::pet_uuid, pets::name, pets::species))
            .filter(pets::id.eq(*pet_id))
            .first(conn)
            .map_err(DomainError::from)?;

    let image_uuid: Option<PetImageUuid> = {
        use crate::schema::pet_images::dsl as pet_images;
        let result: Result<PetImageUuid, _> = pet_images::pet_images
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
