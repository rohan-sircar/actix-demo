use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pets::{
    CreatePet, NewPetTrait, PersonalityTrait, Pet, PetId, PetTrait, PublicPet,
    TraitId, UpdatePet,
};
use crate::models::users::UserId;
use crate::types::DbConnection;

fn fetch_traits_for_pet(
    pet: &Pet,
    conn: &mut DbConnection,
) -> Result<Vec<PetTrait>, DomainError> {
    use crate::schema::personality_traits::dsl as personality_traits;
    use crate::schema::pet_personality_traits::dsl as pet_personality_traits;

    let trait_ids = pet_personality_traits::pet_personality_traits
        .select(pet_personality_traits::trait_id)
        .filter(pet_personality_traits::pet_id.eq(pet.id.as_int()))
        .load::<i32>(conn)?;

    let mut traits = Vec::new();

    for trait_id in trait_ids {
        let trait_record = personality_traits::personality_traits
            .select((personality_traits::id, personality_traits::name))
            .filter(personality_traits::id.eq(trait_id))
            .first::<(TraitId, String)>(conn)
            .optional()?;

        if let Some((id, name)) = trait_record {
            traits.push(PetTrait { id, name });
        }
    }

    Ok(traits)
}

pub fn create_pet(
    user_id: &UserId,
    create: CreatePet,
    conn: &mut DbConnection,
) -> Result<PublicPet, DomainError> {
    use crate::schema::pets::dsl as pets;

    let trait_names = create.traits.clone();

    let pet_id = conn
        .transaction::<_, DomainError, _>(|conn| {
            let new_pet_id = diesel::insert_into(pets::pets)
                .values((
                    pets::user_id.eq(user_id),
                    pets::name.eq(&create.name),
                    pets::species.eq(&create.species),
                    pets::breed.eq(&create.breed),
                    pets::date_of_birth.eq(create.date_of_birth),
                    pets::gender.eq(&create.gender),
                    pets::weight.eq(create.weight),
                    pets::color_markings.eq(&create.color_markings),
                    pets::description.eq(&create.description),
                ))
                .returning(pets::id)
                .get_result::<i32>(conn)?;

            if !trait_names.is_empty() {
                use crate::schema::personality_traits::dsl as personality_traits;
                use crate::schema::pet_personality_traits::dsl as pet_personality_traits;

                let mut inserted_traits = Vec::new();

                for trait_name in &trait_names {
                    let trait_id = personality_traits::personality_traits
                        .select(personality_traits::id)
                        .filter(personality_traits::name.eq(trait_name))
                        .first::<i32>(conn)
                        .optional()?;

                    if let Some(trait_id) = trait_id {
                        inserted_traits.push(NewPetTrait {
                            pet_id: new_pet_id,
                            trait_id,
                        });
                    } else {
                        tracing::warn!(
                            "Trait '{}' not found in personality_traits, skipping",
                            trait_name
                        );
                    }
                }

                if !inserted_traits.is_empty() {
                    diesel::insert_into(pet_personality_traits::pet_personality_traits)
                        .values(&inserted_traits)
                        .execute(conn)?;
                }
            }

            Ok(new_pet_id)
        })?;

    let pet = pets::pets.filter(pets::id.eq(pet_id)).first::<Pet>(conn)?;

    let traits = fetch_traits_for_pet(&pet, conn)?;

    Ok(PublicPet::from((&pet, traits)))
}

pub fn get_pet(
    pet_id: &PetId,
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Option<PublicPet>, DomainError> {
    use crate::schema::pets::dsl as pets;

    let pet = pets::pets
        .filter(pets::id.eq(pet_id))
        .filter(pets::user_id.eq(user_id))
        .first::<Pet>(conn)
        .optional()?;

    match pet {
        Some(pet) => {
            let traits = fetch_traits_for_pet(&pet, conn)?;
            Ok(Some(PublicPet::from((&pet, traits))))
        }
        None => Ok(None),
    }
}

pub fn list_pets(
    user_id: &UserId,
    species: Option<&str>,
    trait_names: Option<Vec<&str>>,
    conn: &mut DbConnection,
) -> Result<Vec<PublicPet>, DomainError> {
    use crate::schema::pets::dsl as pets;

    let mut query = pets::pets.filter(pets::user_id.eq(user_id)).into_boxed();

    if let Some(s) = species {
        query = query.filter(pets::species.ilike(s));
    }

    let pets_list = query.load::<Pet>(conn)?;

    let mut result = Vec::new();

    for pet in pets_list {
        if let Some(ref trait_names) = trait_names {
            if trait_names.is_empty() {
                let traits = fetch_traits_for_pet(&pet, conn)?;
                result.push(PublicPet::from((&pet, traits)));
                continue;
            }

            use crate::schema::personality_traits::dsl as pt_dsl;
            use crate::schema::pet_personality_traits::dsl as ppt_dsl;

            let matching_count = pt_dsl::personality_traits
                .inner_join(ppt_dsl::pet_personality_traits)
                .filter(ppt_dsl::pet_id.eq(pet.id.as_int()))
                .filter(pt_dsl::name.eq_any(trait_names))
                .count()
                .get_result::<i64>(conn)?;

            if matching_count > 0 {
                let traits = fetch_traits_for_pet(&pet, conn)?;
                result.push(PublicPet::from((&pet, traits)));
            }
        } else {
            let traits = fetch_traits_for_pet(&pet, conn)?;
            result.push(PublicPet::from((&pet, traits)));
        }
    }

    Ok(result)
}

pub fn update_pet(
    pet_id: &PetId,
    user_id: &UserId,
    updates: UpdatePet,
    conn: &mut DbConnection,
) -> Result<PublicPet, DomainError> {
    use crate::schema::pets::dsl as pets;

    let pet = pets::pets
        .filter(pets::id.eq(pet_id))
        .filter(pets::user_id.eq(user_id))
        .first::<Pet>(conn)
        .optional()?;

    let pet = match pet {
        Some(p) => p,
        None => {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet {} not found or does not belong to user {}",
                pet_id, user_id
            )))
        }
    };

    let name = updates.name.clone();
    let species = updates.species.clone();
    let breed = updates.breed.clone();
    let date_of_birth = updates.date_of_birth;
    let gender = updates.gender.clone();
    let weight = updates.weight;
    let color_markings = updates.color_markings.clone();
    let description = updates.description.clone();

    let mut updated_name = pet.name.clone();
    let mut updated_species = pet.species.clone();
    let mut updated_breed = pet.breed.clone();
    let mut updated_date_of_birth = pet.date_of_birth;
    let mut updated_gender = pet.gender.clone();
    let mut updated_weight = pet.weight;
    let mut updated_color_markings = pet.color_markings.clone();
    let mut updated_description = pet.description.clone();

    if updates.should_update("name") {
        updated_name = name.unwrap_or(pet.name.clone());
    }
    if updates.should_update("species") {
        updated_species = species.unwrap_or(pet.species.clone());
    }
    if updates.should_update("breed") {
        updated_breed = breed.flatten();
    }
    if updates.should_update("date_of_birth") {
        updated_date_of_birth = date_of_birth.flatten();
    }
    if updates.should_update("gender") {
        updated_gender = gender.flatten();
    }
    if updates.should_update("weight") {
        updated_weight = weight.flatten();
    }
    if updates.should_update("color_markings") {
        updated_color_markings = color_markings.flatten();
    }
    if updates.should_update("description") {
        updated_description = description.flatten();
    }

    diesel::update(pets::pets.filter(pets::id.eq(pet_id)))
        .set((
            pets::name.eq(updated_name),
            pets::species.eq(updated_species),
            pets::breed.eq(updated_breed),
            pets::date_of_birth.eq(updated_date_of_birth),
            pets::gender.eq(updated_gender),
            pets::weight.eq(updated_weight),
            pets::color_markings.eq(updated_color_markings),
            pets::description.eq(updated_description),
        ))
        .execute(conn)?;

    if updates.should_update("traits") {
        use crate::schema::pet_personality_traits::dsl as pet_personality_traits;

        diesel::delete(
            pet_personality_traits::pet_personality_traits
                .filter(pet_personality_traits::pet_id.eq(pet_id.as_int())),
        )
        .execute(conn)?;

        if let Some(ref trait_names) = updates.traits {
            use crate::schema::personality_traits::dsl as personality_traits;

            let mut inserted_traits = Vec::new();

            for trait_name in trait_names {
                let trait_id = personality_traits::personality_traits
                    .select(personality_traits::id)
                    .filter(personality_traits::name.eq(trait_name))
                    .first::<i32>(conn)
                    .optional()?;

                if let Some(trait_id) = trait_id {
                    inserted_traits.push(NewPetTrait {
                        pet_id: pet_id.as_int(),
                        trait_id,
                    });
                } else {
                    tracing::warn!(
                        "Trait '{}' not found in personality_traits, skipping",
                        trait_name
                    );
                }
            }

            if !inserted_traits.is_empty() {
                diesel::insert_into(
                    pet_personality_traits::pet_personality_traits,
                )
                .values(&inserted_traits)
                .execute(conn)?;
            }
        }
    }

    let pet = pets::pets.filter(pets::id.eq(pet_id)).first::<Pet>(conn)?;
    let traits = fetch_traits_for_pet(&pet, conn)?;

    Ok(PublicPet::from((&pet, traits)))
}

pub fn delete_pet(
    pet_id: &PetId,
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<(), DomainError> {
    use crate::schema::pets::dsl as pets;

    let deleted = diesel::delete(
        pets::pets
            .filter(pets::id.eq(pet_id))
            .filter(pets::user_id.eq(user_id)),
    )
    .execute(conn)?;

    if deleted == 0 {
        return Err(DomainError::new_entity_does_not_exist_error(format!(
            "Pet {} not found or does not belong to user {}",
            pet_id, user_id
        )));
    }

    Ok(())
}

pub fn list_traits(
    conn: &mut DbConnection,
) -> Result<Vec<PersonalityTrait>, DomainError> {
    use crate::schema::personality_traits::dsl as personality_traits;

    let traits = personality_traits::personality_traits
        .select((
            personality_traits::id,
            personality_traits::name,
            personality_traits::created_at,
        ))
        .order(personality_traits::id.asc())
        .load::<PersonalityTrait>(conn)?;

    Ok(traits)
}

pub fn get_public_pet(
    pet_id: &PetId,
    conn: &mut DbConnection,
) -> Result<PublicPet, DomainError> {
    use crate::schema::pets::dsl as pets;

    let pet = pets::pets
        .filter(pets::id.eq(pet_id))
        .first::<Pet>(conn)
        .optional()?;

    let pet = match pet {
        Some(p) => p,
        None => {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet {} not found",
                pet_id
            )))
        }
    };

    let traits = fetch_traits_for_pet(&pet, conn)?;

    Ok(PublicPet::from((&pet, traits)))
}
