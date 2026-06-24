use diesel::prelude::*;

use crate::actions::pets::minimal_owner;
use crate::errors::DomainError;
use crate::models::pets::{Pet, PetId, PetTrait, PublicPet};
use crate::models::users::UserId;
use crate::routes::discover::DiscoverPetsQuery;
use crate::types::DbConnection;

/// Returns a single random pet that the user has not yet interacted with.
/// Excludes pets owned by the user and pets already liked/disliked.
pub fn get_next_pet(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Option<PublicPet>, DomainError> {
    use crate::schema::likes::dsl as likes;
    use crate::schema::pets::dsl as pets;

    // Find pets that the user hasn't interacted with yet
    let pet: Result<Pet, _> = pets::pets
        .filter(
            pets::user_id.ne(*user_id).and(
                pets::id.ne_all(
                    likes::likes
                        .select(likes::pet_id)
                        .filter(likes::user_id.eq(*user_id)),
                ),
            ),
        )
        .order(diesel::dsl::sql::<diesel::sql_types::Double>("RANDOM()"))
        .limit(1)
        .first(conn);

    match pet {
        Ok(pet) => {
            let traits = fetch_traits_for_pet(&pet.id, conn)?;
            let primary_image = fetch_primary_image(&pet.id, conn)?;
            let owner = minimal_owner(&pet.user_id, &mut *conn);
            Ok(Some(PublicPet::new(
                &pet,
                traits,
                primary_image.as_ref(),
                owner,
            )))
        }
        Err(diesel::NotFound) => Ok(None),
        Err(e) => Err(DomainError::from(e)),
    }
}

/// Returns a paginated list of discoverable pets with optional filters.
/// Excludes pets owned by the user and pets already liked/disliked.
pub fn list_discoverable_pets(
    user_id: &UserId,
    query: &DiscoverPetsQuery,
    conn: &mut DbConnection,
) -> Result<(Vec<PublicPet>, i64), DomainError> {
    use crate::schema::likes::dsl as likes;
    use crate::schema::pets::dsl as pets;

    let build_query = || {
        let mut q = pets::pets
            .filter(
                pets::user_id.ne(*user_id).and(
                    pets::id.ne_all(
                        likes::likes
                            .select(likes::pet_id)
                            .filter(likes::user_id.eq(*user_id)),
                    ),
                ),
            )
            .into_boxed();

        if let Some(species_filter) = &query.species {
            q = q.filter(
                pets::species.ilike(format!("%{}%", species_filter.inner())),
            );
        }

        if let Some(gender_filter) = &query.gender {
            q = q.filter(pets::gender.eq(gender_filter.clone()));
        }

        // Age filtering using date_of_birth
        if let Some(min_age) = query.age_min {
            let max_dob = chrono::Local::now()
                .naive_local()
                .date()
                .checked_sub_signed(chrono::Duration::days(
                    (min_age * 365.25) as i64,
                ))
                .unwrap_or(chrono::NaiveDate::MIN);
            q = q.filter(pets::date_of_birth.le(max_dob));
        }

        if let Some(max_age) = query.age_max {
            let min_dob = chrono::Local::now()
                .naive_local()
                .date()
                .checked_sub_signed(chrono::Duration::days(
                    (max_age * 365.25) as i64,
                ))
                .unwrap_or(chrono::NaiveDate::MAX);
            q = q.filter(pets::date_of_birth.ge(min_dob));
        }

        q
    };

    let total_count = build_query().count().get_result::<i64>(conn)?;

    let pet_results: Vec<Pet> = build_query()
        .order(pets::created_at.desc())
        .offset(query.offset.as_uint().into())
        .limit(query.limit.as_uint().into())
        .load(conn)?;

    let mut public_pets = Vec::new();
    for pet in pet_results {
        let traits = fetch_traits_for_pet(&pet.id, conn)?;
        let primary_image = fetch_primary_image(&pet.id, conn)?;
        let owner = minimal_owner(&pet.user_id, &mut *conn);
        public_pets.push(PublicPet::new(
            &pet,
            traits,
            primary_image.as_ref(),
            owner,
        ));
    }

    Ok((public_pets, total_count))
}

fn fetch_traits_for_pet(
    pet_id: &PetId,
    conn: &mut DbConnection,
) -> Result<Vec<PetTrait>, DomainError> {
    use crate::models::pets::TraitId;
    use crate::schema::personality_traits::dsl as personality_traits;
    use crate::schema::pet_personality_traits::dsl as pet_personality_traits;

    let results: Vec<(TraitId, String)> =
        pet_personality_traits::pet_personality_traits
            .inner_join(personality_traits::personality_traits)
            .filter(pet_personality_traits::pet_id.eq(*pet_id))
            .select((personality_traits::id, personality_traits::name))
            .load(conn)?;

    Ok(results
        .into_iter()
        .map(|(id, name)| PetTrait { id, name })
        .collect())
}

fn fetch_primary_image(
    pet_id: &PetId,
    conn: &mut DbConnection,
) -> Result<Option<crate::models::pets::PetImage>, DomainError> {
    use crate::schema::pet_images::dsl as pet_images;

    let image: Result<crate::models::pets::PetImage, _> =
        pet_images::pet_images
            .filter(
                pet_images::pet_id
                    .eq(*pet_id)
                    .and(pet_images::is_primary.eq(true)),
            )
            .first(conn);

    match image {
        Ok(img) => Ok(Some(img)),
        Err(diesel::NotFound) => Ok(None),
        Err(e) => Err(DomainError::from(e)),
    }
}
