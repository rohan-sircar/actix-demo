use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::likes::{
    CreateLike, Like, LikeDirection, LikeResponse, LikeWithPet, Match, NewLike,
    NewMatch,
};
use crate::models::pets::{PetId, PetImageUuid, PetName, PetSpecies, PetUuid};
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
        if direction == LikeDirection::Like {
            use crate::schema::matches::dsl as matches;
            use crate::schema::pets::dsl::{
                id as pet_id_col, pets, user_id as owner_col,
            };

            // Find a reciprocal like from the pet owner (one of their pets liked by current user)
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

            if let Ok(recip) = reciprocal {
                // Determine which like_id is smaller for the constraint like_id_a < like_id_b
                let (a, b) = if recip.id.as_int() < inserted_id {
                    (recip.id.as_int(), inserted_id)
                } else {
                    (inserted_id, recip.id.as_int())
                };

                // Insert into matches table
                let new_match = NewMatch::new(
                    crate::models::likes::LikeId::try_from(a as u32).unwrap(),
                    crate::models::likes::LikeId::try_from(b as u32).unwrap(),
                );
                diesel::insert_into(matches::matches)
                    .values(&new_match)
                    .execute(conn)?;
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
/// Returns Some(direction) if interaction exists, None otherwise.
pub fn get_pet_interaction(
    user_id: &UserId,
    pet_uuid: &PetUuid,
    conn: &mut DbConnection,
) -> Result<Option<LikeDirection>, DomainError> {
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

    Ok(interaction.map(|like| like.direction))
}

/// Returns mutual matches with both pets involved for the current user.
pub fn list_matches_with_pets(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<crate::models::likes::MatchWithPets>, DomainError> {
    use crate::schema::likes::dsl as likes;
    use crate::schema::matches::dsl as matches;

    // Load all matches ordered by matched_at
    let all_matches: Vec<Match> = matches::matches
        .order(matches::matched_at.desc())
        .load(conn)?;

    let mut result = Vec::new();

    for match_row in all_matches {
        // Fetch both likes
        let like_a: Result<Like, _> = likes::likes
            .filter(likes::id.eq(match_row.like_id_a))
            .first(conn);

        let like_b: Result<Like, _> = likes::likes
            .filter(likes::id.eq(match_row.like_id_b))
            .first(conn);

        let (current_like, other_like) = match (like_a, like_b) {
            (Ok(a), Ok(b)) if a.user_id == *user_id => (a, b),
            (Ok(a), Ok(b)) if b.user_id == *user_id => (b, a),
            _ => continue, // Current user not involved in this match
        };

        let other_user_id = other_like.pet_owner_id;

        // The pet the current user liked (other person's pet)
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

        // The pet the other person liked (current user's pet)
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
            is_match: true,
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
