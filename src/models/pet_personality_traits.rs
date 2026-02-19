use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{models::pet_basic_info::*, schema::pet_personality_traits};

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Selectable, Identifiable,
)]
#[diesel(table_name = pet_personality_traits)]
#[diesel(check_for_backend(Pg))]
pub struct PetPersonalityTraits {
    pub id: i32,
    pub pet_basic_info_id: PetBasicInfoId,
    pub bio: Option<String>,
    pub personality_traits: Option<Vec<Option<String>>>,
    pub good_with_dogs: Option<bool>,
    pub good_with_cats: Option<bool>,
    pub good_with_kids: Option<bool>,
    pub house_trained: Option<bool>,
    pub vaccinated: Option<bool>,
    pub spayed_neutered: Option<bool>,
    pub microchipped: Option<bool>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_personality_traits)]
pub struct NewPetPersonalityTraits {
    pub pet_basic_info_id: PetBasicInfoId,
    pub bio: Option<String>,
    pub personality_traits: Option<Vec<Option<String>>>,
    pub good_with_dogs: Option<bool>,
    pub good_with_cats: Option<bool>,
    pub good_with_kids: Option<bool>,
    pub house_trained: Option<bool>,
    pub vaccinated: Option<bool>,
    pub spayed_neutered: Option<bool>,
    pub microchipped: Option<bool>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = pet_personality_traits)]
pub struct UpdatePetPersonalityTraits {
    pub bio: Option<Option<String>>,
    pub personality_traits: Option<Option<Vec<Option<String>>>>,
    pub good_with_dogs: Option<Option<bool>>,
    pub good_with_cats: Option<Option<bool>>,
    pub good_with_kids: Option<Option<bool>>,
    pub house_trained: Option<Option<bool>>,
    pub vaccinated: Option<Option<bool>>,
    pub spayed_neutered: Option<Option<bool>>,
    pub microchipped: Option<Option<bool>>,
}
