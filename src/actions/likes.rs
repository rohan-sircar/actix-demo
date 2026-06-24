use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::likes::{
    CreateLike, Like, LikeDirection, LikeResponse, NewLike,
};
use crate::models::pets::PetId;
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
