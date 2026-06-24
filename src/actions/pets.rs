use actix_web::web;
use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pets::{
    CreatePet, ImageId, NewPetTrait, PersonalityTrait, Pet, PetImage, PetTrait,
    PetUuid, PublicPet, PublicPetImage, PublicPetOwner, TraitId, UpdatePet,
};
use crate::models::users::UserId;
use crate::types::DbConnection;
use crate::types::DbPool;
use crate::utils::images::resize_and_encode_webp;

fn fetch_all_traits_for_pets(
    pet_ids: &[i32],
    conn: &mut DbConnection,
) -> Result<std::collections::HashMap<i32, Vec<PetTrait>>, DomainError> {
    use crate::schema::personality_traits::dsl as personality_traits;
    use crate::schema::pet_personality_traits::dsl as pet_personality_traits;

    let results = pet_personality_traits::pet_personality_traits
        .inner_join(personality_traits::personality_traits)
        .filter(pet_personality_traits::pet_id.eq_any(pet_ids))
        .select((
            pet_personality_traits::pet_id,
            personality_traits::id,
            personality_traits::name,
        ))
        .load::<(i32, TraitId, String)>(conn)?;

    let mut map = std::collections::HashMap::new();
    for (pet_id, trait_id, name) in results {
        map.entry(pet_id)
            .or_insert_with(Vec::new)
            .push(PetTrait { id: trait_id, name });
    }

    Ok(map)
}

pub fn create_pet(
    user_id: &UserId,
    create: CreatePet,
    conn: &mut DbConnection,
) -> Result<PublicPet, DomainError> {
    use crate::schema::pets::dsl as pets;

    let pet_uuid = uuid::Uuid::new_v4();
    let trait_names = create.traits.clone();

    let pet_id = conn
        .transaction::<_, DomainError, _>(|conn| {
            let new_pet_id = diesel::insert_into(pets::pets)
                .values((
                    pets::pet_uuid.eq(&pet_uuid),
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

    let traits = fetch_all_traits_for_pets(&[pet_id], conn)?;
    let traits = traits.get(&pet_id).cloned().unwrap_or_default();

    let primary_image = fetch_primary_image(pet_id, conn)?;

    let owner = minimal_owner(&pet.user_id, conn);
    Ok(PublicPet::new(&pet, traits, primary_image.as_ref(), owner))
}

pub fn get_pet(
    pet_uuid: &PetUuid,
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Option<PublicPet>, DomainError> {
    use crate::schema::pets::dsl as pets;

    let pet = pets::pets
        .filter(pets::pet_uuid.eq(pet_uuid))
        .filter(pets::user_id.eq(user_id))
        .first::<Pet>(conn)
        .optional()?;

    match pet {
        Some(pet) => {
            let traits = fetch_all_traits_for_pets(&[pet.id.as_int()], conn)?;
            let traits =
                traits.get(&pet.id.as_int()).cloned().unwrap_or_default();
            let primary_image = fetch_primary_image(pet.id.as_int(), conn)?;
            let owner = minimal_owner(&pet.user_id, conn);
            Ok(Some(PublicPet::new(
                &pet,
                traits,
                primary_image.as_ref(),
                owner,
            )))
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

    if pets_list.is_empty() {
        return Ok(Vec::new());
    }

    let mut filtered_pets = pets_list;

    if let Some(ref trait_names) = trait_names {
        if !trait_names.is_empty() {
            use crate::schema::personality_traits::dsl as pt_dsl;
            use crate::schema::pet_personality_traits::dsl as ppt_dsl;

            let matching_pet_ids = ppt_dsl::pet_personality_traits
                .inner_join(pt_dsl::personality_traits)
                .filter(pt_dsl::name.eq_any(trait_names))
                .select(ppt_dsl::pet_id)
                .distinct()
                .load::<i32>(conn)?;

            let matching_set: std::collections::HashSet<i32> =
                matching_pet_ids.into_iter().collect();
            filtered_pets.retain(|pet| matching_set.contains(&pet.id.as_int()));
        }
    }

    if filtered_pets.is_empty() {
        return Ok(Vec::new());
    }

    let pet_ids: Vec<i32> =
        filtered_pets.iter().map(|p| p.id.as_int()).collect();
    let traits_map = fetch_all_traits_for_pets(&pet_ids, conn)?;

    let mut result = Vec::new();
    for pet in filtered_pets {
        let traits = traits_map
            .get(&pet.id.as_int())
            .cloned()
            .unwrap_or_default();
        let primary_image = fetch_primary_image(pet.id.as_int(), conn)?;
        let owner = minimal_owner(&pet.user_id, conn);
        result.push(PublicPet::new(
            &pet,
            traits,
            primary_image.as_ref(),
            owner,
        ));
    }

    Ok(result)
}

pub fn update_pet(
    pet_uuid: &PetUuid,
    user_id: &UserId,
    updates: UpdatePet,
    conn: &mut DbConnection,
) -> Result<PublicPet, DomainError> {
    use crate::schema::pets::dsl as pets;

    let name = updates.name.clone();
    let species = updates.species.clone();
    let breed = updates.breed.clone();
    let date_of_birth = updates.date_of_birth;
    let gender = updates.gender.clone();
    let weight = updates.weight;
    let color_markings = updates.color_markings.clone();
    let description = updates.description.clone();

    let pet_id = conn
        .transaction::<_, DomainError, _>(|conn| {
            let pet = pets::pets
                .filter(pets::pet_uuid.eq(pet_uuid))
                .filter(pets::user_id.eq(user_id))
                .first::<Pet>(conn)
                .optional()?
                .ok_or_else(|| {
                    DomainError::new_entity_does_not_exist_error(format!(
                        "Pet {} not found or does not belong to user {}",
                        pet_uuid, user_id
                    ))
                })?;

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

            diesel::update(pets::pets.filter(pets::pet_uuid.eq(pet_uuid)))
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
                        .filter(pet_personality_traits::pet_id.eq(pet.id.as_int())),
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
                                pet_id: pet.id.as_int(),
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

            Ok(pet.id.as_int())
        })?;

    let pet = pets::pets.filter(pets::id.eq(pet_id)).first::<Pet>(conn)?;
    let traits = fetch_all_traits_for_pets(&[pet_id], conn)?;
    let traits = traits.get(&pet_id).cloned().unwrap_or_default();

    let primary_image = fetch_primary_image(pet_id, conn)?;

    let owner = minimal_owner(&pet.user_id, conn);
    Ok(PublicPet::new(&pet, traits, primary_image.as_ref(), owner))
}

pub fn delete_pet(
    pet_uuid: &PetUuid,
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<(), DomainError> {
    use crate::schema::pets::dsl as pets;

    let deleted = diesel::delete(
        pets::pets
            .filter(pets::pet_uuid.eq(pet_uuid))
            .filter(pets::user_id.eq(user_id)),
    )
    .execute(conn)?;

    if deleted == 0 {
        return Err(DomainError::new_entity_does_not_exist_error(format!(
            "Pet {} not found or does not belong to user {}",
            pet_uuid, user_id
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
    pet_uuid: &PetUuid,
    conn: &mut DbConnection,
) -> Result<PublicPet, DomainError> {
    use crate::schema::pets::dsl as pets;

    let pet = pets::pets
        .filter(pets::pet_uuid.eq(pet_uuid))
        .first::<Pet>(conn)
        .optional()?;

    let pet = match pet {
        Some(p) => p,
        None => {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet {} not found",
                pet_uuid
            )))
        }
    };

    let traits = fetch_all_traits_for_pets(&[pet.id.as_int()], conn)?;
    let traits = traits.get(&pet.id.as_int()).cloned().unwrap_or_default();

    let primary_image = fetch_primary_image(pet.id.as_int(), conn)?;

    let owner = fetch_owner_info(pet.user_id, conn)
        .unwrap_or_else(|| minimal_owner(&pet.user_id, conn));

    Ok(PublicPet::new(&pet, traits, primary_image.as_ref(), owner))
}

fn fetch_owner_info(
    user_id: crate::models::users::UserId,
    conn: &mut DbConnection,
) -> Option<PublicPetOwner> {
    use crate::models::users::UserUuid;
    use crate::schema::profiles::dsl as profiles;
    use crate::schema::users::dsl as users;

    // Fetch profile info
    let (user_uuid_opt, display_name, avatar_url) = users::users
        .inner_join(profiles::profiles)
        .filter(users::id.eq(user_id))
        .select((
            users::user_uuid,
            profiles::display_name,
            profiles::avatar_url,
        ))
        .first::<(uuid::Uuid, Option<String>, Option<String>)>(conn)
        .ok()?;

    // Count pets
    use crate::schema::pets::dsl as pets;
    let pets_owned: i64 = pets::pets
        .filter(pets::user_id.eq(user_id))
        .count()
        .get_result(conn)
        .ok()?;

    Some(PublicPetOwner {
        user_uuid: UserUuid::try_from(user_uuid_opt.to_string()).ok()?,
        display_name,
        avatar_url,
        pets_owned: pets_owned as u32,
    })
}

pub(crate) fn minimal_owner(
    user_id: &crate::models::users::UserId,
    conn: &mut DbConnection,
) -> PublicPetOwner {
    use crate::models::users::UserUuid;
    use crate::schema::users::dsl as users;

    let user_uuid = users::users
        .select(users::user_uuid)
        .filter(users::id.eq(user_id))
        .first::<uuid::Uuid>(conn)
        .unwrap_or_else(|_| uuid::Uuid::nil());

    PublicPetOwner {
        user_uuid: UserUuid::try_from(user_uuid.to_string()).unwrap_or_else(
            |_| {
                UserUuid::try_from(
                    "00000000-0000-0000-0000-000000000000".to_string(),
                )
                .unwrap()
            },
        ),
        display_name: None,
        avatar_url: None,
        pets_owned: 0,
    }
}

fn fetch_primary_image(
    pet_id: i32,
    conn: &mut DbConnection,
) -> Result<Option<PetImage>, DomainError> {
    use crate::schema::pet_images::dsl as pet_images;

    let image = pet_images::pet_images
        .filter(pet_images::pet_id.eq(pet_id))
        .filter(pet_images::is_primary.eq(true))
        .first::<PetImage>(conn)
        .optional()?;

    Ok(image)
}

pub fn upload_pet_image(
    pet_uuid: &PetUuid,
    user_id: &UserId,
    image_bytes: Vec<u8>,
    pool: &DbPool,
    _bucket_name: &str,
) -> Result<(PublicPetImage, web::Bytes, web::Bytes, web::Bytes), DomainError> {
    use crate::schema::pet_images::dsl as pet_images;
    use crate::schema::pets::dsl as pets;

    let mut conn = pool.get().map_err(|e| {
        DomainError::new_internal_error(format!(
            "Failed to get DB connection: {}",
            e
        ))
    })?;

    let resized = resize_and_encode_webp(&image_bytes).map_err(|e| {
        DomainError::new_bad_input_error(format!(
            "Image processing failed: {}",
            e
        ))
    })?;

    let new_image_uuid = uuid::Uuid::new_v4();

    let (image_id, is_primary, sort_order) = conn
        .transaction::<_, DomainError, _>(|conn| {
            let pet = pets::pets
                .filter(pets::pet_uuid.eq(pet_uuid))
                .filter(pets::user_id.eq(user_id))
                .first::<Pet>(conn)
                .optional()?
                .ok_or_else(|| {
                    DomainError::new_entity_does_not_exist_error(format!(
                        "Pet {} not found",
                        pet_uuid
                    ))
                })?;

            let is_first_image: i64 = pet_images::pet_images
                .filter(pet_images::pet_id.eq(pet.id.as_int()))
                .count()
                .get_result(conn)?;

            let sort_order = if is_first_image == 0 {
                0
            } else {
                pet_images::pet_images
                    .filter(pet_images::pet_id.eq(pet.id.as_int()))
                    .select(pet_images::sort_order)
                    .order(pet_images::sort_order.desc())
                    .first::<i32>(conn)?
                    + 1
            };

            let is_primary = is_first_image == 0;

            let id: i32 = diesel::insert_into(pet_images::pet_images)
                .values((
                    pet_images::uuid.eq(new_image_uuid),
                    pet_images::pet_id.eq(pet.id.as_int()),
                    pet_images::thumbnail_key.eq(format!(
                        "pets/{}/{}/thumbnail.webp",
                        pet_uuid, new_image_uuid
                    )),
                    pet_images::medium_key.eq(format!(
                        "pets/{}/{}/medium.webp",
                        pet_uuid, new_image_uuid
                    )),
                    pet_images::original_key.eq(format!(
                        "pets/{}/{}/original.webp",
                        pet_uuid, new_image_uuid
                    )),
                    pet_images::format.eq("webp"),
                    pet_images::is_primary.eq(is_primary),
                    pet_images::sort_order.eq(sort_order),
                ))
                .returning(pet_images::id)
                .get_result(conn)
                .map_err(|e| {
                    DomainError::new_internal_error(format!(
                        "DB insert failed: {}",
                        e
                    ))
                })?;

            Ok((id, is_primary, sort_order))
        })?;

    Ok((
        PublicPetImage {
            id: ImageId::try_from(image_id as u32).map_err(|e| {
                DomainError::new_internal_error(format!(
                    "Invalid image ID: {}",
                    e
                ))
            })?,
            uuid: new_image_uuid,
            format: "webp".to_string(),
            is_primary,
            sort_order,
            created_at: chrono::Utc::now().naive_utc(),
        },
        resized.thumbnail,
        resized.medium,
        resized.original,
    ))
}

pub fn list_pet_images(
    pet_uuid: &PetUuid,
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<PublicPetImage>, DomainError> {
    use crate::schema::pet_images::dsl as pet_images;
    use crate::schema::pets::dsl as pets;

    let pet = pets::pets
        .filter(pets::pet_uuid.eq(pet_uuid))
        .filter(pets::user_id.eq(user_id))
        .first::<Pet>(conn)
        .optional()?;

    let pet = match pet {
        Some(p) => p,
        None => {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Pet {} not found",
                pet_uuid
            )))
        }
    };

    let images = pet_images::pet_images
        .filter(pet_images::pet_id.eq(pet.id.as_int()))
        .order(pet_images::sort_order.asc())
        .load::<PetImage>(conn)?;

    Ok(images
        .into_iter()
        .map(|img| PublicPetImage::from(&img))
        .collect())
}

pub fn delete_pet_image(
    pet_uuid: &PetUuid,
    image_uuid: &uuid::Uuid,
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<(String, String, String), DomainError> {
    use crate::schema::pet_images::dsl as pet_images;
    use crate::schema::pets::dsl as pets;

    let keys = conn.transaction::<_, DomainError, _>(|conn| {
        let pet = pets::pets
            .filter(pets::pet_uuid.eq(pet_uuid))
            .filter(pets::user_id.eq(user_id))
            .first::<Pet>(conn)
            .optional()?
            .ok_or_else(|| {
                DomainError::new_entity_does_not_exist_error(format!(
                    "Pet {} not found",
                    pet_uuid
                ))
            })?;

        let image = pet_images::pet_images
            .filter(pet_images::pet_id.eq(pet.id.as_int()))
            .filter(pet_images::uuid.eq(*image_uuid))
            .first::<PetImage>(conn)
            .optional()?
            .ok_or_else(|| {
                DomainError::new_entity_does_not_exist_error(format!(
                    "Image {} not found",
                    image_uuid
                ))
            })?;

        let thumbnail_key = image.thumbnail_key.clone();
        let medium_key = image.medium_key.clone();
        let original_key = image.original_key.clone();

        diesel::delete(
            pet_images::pet_images.filter(pet_images::id.eq(image.id.as_int())),
        )
        .execute(conn)?;

        diesel::update(
            pet_images::pet_images
                .filter(pet_images::pet_id.eq(pet.id.as_int()))
                .filter(pet_images::sort_order.gt(image.sort_order)),
        )
        .set(pet_images::sort_order.eq(pet_images::sort_order - 1))
        .execute(conn)?;

        Ok((thumbnail_key, medium_key, original_key))
    })?;

    Ok(keys)
}

pub fn set_primary_image(
    pet_uuid: &PetUuid,
    image_uuid: &uuid::Uuid,
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<PublicPetImage, DomainError> {
    use crate::schema::pet_images::dsl as pet_images;
    use crate::schema::pets::dsl as pets;

    conn.transaction::<_, DomainError, _>(|conn| {
        let pet = pets::pets
            .filter(pets::pet_uuid.eq(pet_uuid))
            .filter(pets::user_id.eq(user_id))
            .first::<Pet>(conn)
            .optional()?
            .ok_or_else(|| {
                DomainError::new_entity_does_not_exist_error(format!(
                    "Pet {} not found",
                    pet_uuid
                ))
            })?;

        let image = pet_images::pet_images
            .filter(pet_images::pet_id.eq(pet.id.as_int()))
            .filter(pet_images::uuid.eq(*image_uuid))
            .first::<PetImage>(conn)
            .optional()?
            .ok_or_else(|| {
                DomainError::new_entity_does_not_exist_error(format!(
                    "Image {} not found",
                    image_uuid
                ))
            })?;

        diesel::update(
            pet_images::pet_images
                .filter(pet_images::pet_id.eq(pet.id.as_int())),
        )
        .set(pet_images::is_primary.eq(false))
        .execute(conn)?;

        diesel::update(
            pet_images::pet_images.filter(pet_images::id.eq(image.id.as_int())),
        )
        .set(pet_images::is_primary.eq(true))
        .execute(conn)?;

        let updated_image = pet_images::pet_images
            .filter(pet_images::id.eq(image.id.as_int()))
            .first::<PetImage>(conn)?;

        Ok(PublicPetImage::from(&updated_image))
    })
}
