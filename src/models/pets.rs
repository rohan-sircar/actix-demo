use crate::models::pet_enums::*;
use bigdecimal::BigDecimal;
use derive_more::Display;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use validators::Validator;

use crate::schema::pet_location_owner;
use crate::utils::regex;
use crate::{models::users::UserId, schema::pet_basic_info};
use validators::prelude::*;

#[derive(
    Debug,
    Display,
    Clone,
    Deserialize,
    Serialize,
    DieselNewType,
    Eq,
    PartialEq,
    Hash,
)]
pub struct PetProfileId(i32);

impl PetProfileId {
    pub fn as_uint(&self) -> u32 {
        self.0.try_into().unwrap()
    }
}

#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq)]
#[validator(regex(regex(regex::PETNAME_REG)))]
pub struct Petname(String);
impl Petname {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq)]
#[validator(regex(regex(regex::PETNAME_REG)))]
pub struct Breedname(String);
impl Breedname {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Selectable, Identifiable,
)]
#[diesel(table_name = pet_basic_info)]
pub struct PetBasicInfo {
    pub id: PetProfileId,
    pub user_id: UserId,
    pub pet_name: Petname,
    pub pet_type: PetType,
    pub breed: Breedname,
    pub age: i32,
    pub weight_kg: f32,
    pub gender: GenderType,
    pub size: Option<SizeType>,
    pub color: Option<String>,
    pub coat_type: Option<CoatType>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_basic_info)]
pub struct NewPetBasicInfo {
    pub user_id: UserId,
    pub pet_name: Petname,
    pub pet_type: PetType,
    pub breed: Breedname,
    pub age: i32,
    pub weight_kg: f32,
    pub gender: GenderType,
    pub size: Option<SizeType>,
    pub color: Option<String>,
    pub coat_type: Option<CoatType>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = pet_basic_info)]
pub struct UpdatePetBasicInfo {
    pub pet_name: Option<Petname>,
    pub pet_type: Option<PetType>,
    pub breed: Option<Breedname>,
    pub age: Option<i32>,
    pub weight_kg: Option<f32>,
    pub gender: Option<GenderType>,
    pub size: Option<Option<SizeType>>,
    pub color: Option<Option<String>>,
    pub coat_type: Option<Option<CoatType>>,
}

use crate::schema::pet_activities;

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Selectable,
)]
#[diesel(table_name = pet_activities)]
#[diesel(check_for_backend(Pg))]
pub struct PetActivities {
    pub id: i32,
    pub pet_profile_id: PetProfileId,
    pub favorite_activities: Option<Vec<Option<String>>>,
    pub likes: Option<Vec<Option<String>>>,
    pub dislikes: Option<Vec<Option<String>>>,
    pub energy_level: Option<EnergyLevelType>,
    pub trainability: Option<TrainabilityType>,
    pub barking_level: Option<BarkingLevelType>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_activities)]
pub struct NewPetActivities {
    pub pet_profile_id: PetProfileId,
    pub favorite_activities: Option<Vec<String>>,
    pub likes: Option<Vec<String>>,
    pub dislikes: Option<Vec<String>>,
    pub energy_level: Option<EnergyLevelType>,
    pub trainability: Option<TrainabilityType>,
    pub barking_level: Option<BarkingLevelType>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = pet_activities)]
pub struct UpdatePetActivities {
    pub favorite_activities: Option<Option<Vec<String>>>,
    pub likes: Option<Option<Vec<String>>>,
    pub dislikes: Option<Option<Vec<String>>>,
    pub energy_level: Option<Option<EnergyLevelType>>,
    pub trainability: Option<Option<TrainabilityType>>,
    pub barking_level: Option<Option<BarkingLevelType>>,
}

use crate::schema::pet_adoption_details;

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_adoption_details)]
pub struct NewPetAdoptionDetails {
    pub pet_profile_id: PetProfileId,
    pub special_needs: bool,
    pub special_needs_description: Option<String>,
    pub adoption_status: Option<AdoptionStatusType>,
    pub shelter_name: Option<String>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = pet_adoption_details)]
pub struct UpdatePetAdoptionDetails {
    pub special_needs: Option<bool>,
    pub special_needs_description: Option<Option<String>>,
    pub adoption_status: Option<Option<AdoptionStatusType>>,
    pub shelter_name: Option<Option<String>>,
}

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::AdoptionStatusType"]
pub enum AdoptionStatusType {
    Adoptable,
    Foster,
    Available,
}

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Selectable,
)]
#[diesel(table_name = pet_adoption_details)]
#[diesel(check_for_backend(Pg))]
pub struct PetAdoptionDetails {
    pub id: i32,
    pub pet_profile_id: PetProfileId,
    pub special_needs: Option<bool>,
    pub special_needs_description: Option<String>,
    pub adoption_status: Option<AdoptionStatusType>,
    pub shelter_name: Option<String>,
}

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Selectable,
)]
#[diesel(table_name = pet_location_owner)]
pub struct PetLocationOwner {
    pub id: i32,
    pub pet_profile_id: PetProfileId,
    pub owner_name: String,
    pub location: String,
    pub address: Option<String>,
    pub lat: Option<BigDecimal>,
    pub lng: Option<BigDecimal>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_location_owner)]
pub struct NewPetLocationOwner {
    pub pet_profile_id: PetProfileId,
    pub owner_name: String,
    pub location: String,
    pub address: Option<String>,
    pub lat: Option<BigDecimal>,
    pub lng: Option<BigDecimal>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = pet_location_owner)]
pub struct UpdatePetLocationOwner {
    pub owner_name: Option<String>,
    pub location: Option<String>,
    pub address: Option<Option<String>>,
    pub lat: Option<Option<BigDecimal>>,
    pub lng: Option<Option<BigDecimal>>,
}

use crate::schema::pet_personality_traits;

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Selectable, Identifiable,
)]
#[diesel(table_name = pet_personality_traits)]
#[diesel(check_for_backend(Pg))]
pub struct PetPersonalityTraits {
    pub id: i32,
    pub pet_profile_id: PetProfileId,
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
    pub pet_profile_id: PetProfileId,
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
