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
                .set(likes::is_match.eq(true))
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
        .filter(likes::user_id.eq(*user_id).and(likes::direction.eq(LikeDirection::Like)))
        .order(likes::created_at.desc())
        .load(conn)?;

    like_rows
        .into_iter()
        .map(|like| {
            let pet = fetch_pet_with_image(&like.pet_id, conn);
            match pet {
                Ok((pet_uuid, pet_name, species, image_uuid)) => Ok(LikeWithPet {
                    pet_uuid,
                    pet_name,
                    species,
                    primary_image_uuid: image_uuid.map(|u| u.to_string()),
                    is_match: like.is_match,
                    created_at: like.created_at,
                }),
                Err(_) => Err(DomainError::new_internal_error(
                    format!("Pet not found for like id {}", like.id),
                )),
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
        .filter(likes::pet_owner_id.eq(*user_id).and(likes::direction.eq(LikeDirection::Like)))
        .order(likes::created_at.desc())
        .load(conn)?;

    like_rows
        .into_iter()
        .map(|like| {
            let pet = fetch_pet_with_image(&like.pet_id, conn);
            match pet {
                Ok((pet_uuid, pet_name, species, image_uuid)) => Ok(LikeWithPet {
                    pet_uuid,
                    pet_name,
                    species,
                    primary_image_uuid: image_uuid.map(|u| u.to_string()),
                    is_match: like.is_match,
                    created_at: like.created_at,
                }),
                Err(_) => Err(DomainError::new_internal_error(
                    format!("Pet not found for like id {}", like.id),
                )),
            }
        })
        .collect()
}

fn fetch_pet_with_image(
    pet_id: &PetId,
    conn: &mut DbConnection,
) -> Result<(PetUuid, String, String, Option<uuid::Uuid>), DomainError> {
    use crate::schema::pets::dsl as pets;

    let (pet_uuid_raw, pet_name, species): (uuid::Uuid, String, String) = pets::pets
        .select((pets::pet_uuid, pets::name, pets::species))
        .filter(pets::id.eq(*pet_id))
        .first(conn)
        .map_err(DomainError::from)?;

    let pet_uuid: PetUuid = PetUuid::try_from(
        pet_uuid_raw.to_string(),
    ).map_err(|e: String| DomainError::new_internal_error(format!("Invalid pet UUID: {}", e)))?;

    let image_uuid: Option<uuid::Uuid> = {
        use crate::schema::pet_images::dsl as pet_images;
        let result: Result<uuid::Uuid, _> = pet_images::pet_images
            .select(pet_images::uuid)
            .filter(
                pet_images::pet_id.eq(*pet_id).and(pet_images::is_primary.eq(true)),
            )
            .first(conn);
        result.ok()
    };

    Ok((pet_uuid, pet_name, species, image_uuid))
}
