use crate::models::pet_enums::*;
use bigdecimal::BigDecimal;
use derive_more::Display;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;
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

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

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
pub struct PetProfileUuid(Uuid);

impl PetProfileUuid {
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl FromStr for PetProfileUuid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(PetProfileUuid).map_err(|e| e.to_string())
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

/// Weight in kilograms, validated to be >= 1.0 and <= 150.0
#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    DieselNewType,
    PartialEq,
    PartialOrd,
    Validator,
)]
#[validator(number(nan(Disallow), range(Inside(min = 1.0, max = 150.0))))]
pub struct WeightKg(f32);

impl WeightKg {
    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

/// Age in years, validated to be >= 1 and <= 30
#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    DieselNewType,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Validator,
)]
#[validator(signed_integer(range(Inside(min = 1, max = 30))))]
pub struct PetAge(pub i32);

impl PetAge {
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Selectable, Identifiable,
)]
#[diesel(table_name = pet_basic_info)]
pub struct PetBasicInfo {
    pub id: PetProfileId,
    pub uuid: PetProfileUuid,
    pub user_id: UserId,
    pub pet_name: Petname,
    pub pet_type: PetType,
    pub breed: Breedname,
    pub age: i32,
    pub weight_kg: WeightKg,
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
    pub weight_kg: WeightKg,
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
    pub age: Option<PetAge>,
    pub weight_kg: Option<WeightKg>,
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
    pub pet_profile_uuid: PetProfileUuid,
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
    pub pet_profile_uuid: PetProfileUuid,
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
    pub pet_profile_uuid: PetProfileUuid,
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
    Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Selectable,
)]
#[diesel(table_name = pet_adoption_details)]
#[diesel(check_for_backend(Pg))]
pub struct PetAdoptionDetails {
    pub id: i32,
    pub pet_profile_uuid: PetProfileUuid,
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
    pub pet_profile_uuid: PetProfileUuid,
    pub owner_name: String,
    pub location: String,
    pub address: Option<String>,
    pub lat: Option<BigDecimal>,
    pub lng: Option<BigDecimal>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_location_owner)]
pub struct NewPetLocationOwner {
    pub pet_profile_uuid: PetProfileUuid,
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
    pub pet_profile_uuid: PetProfileUuid,
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
    pub pet_profile_uuid: PetProfileUuid,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pet_age_valid() {
        // Test valid ages within range (1-30)
        assert!(PetAge::parse_i32(1).is_ok());
        assert!(PetAge::parse_i32(15).is_ok());
        assert!(PetAge::parse_i32(30).is_ok());
    }

    #[test]
    fn test_pet_age_invalid() {
        // Test invalid ages outside range
        assert!(PetAge::parse_i32(0).is_err());
        assert!(PetAge::parse_i32(-1).is_err());
        assert!(PetAge::parse_i32(31).is_err());
        assert!(PetAge::parse_i32(100).is_err());
    }

    #[test]
    fn test_pet_age_parse_string() {
        // Test parsing from string
        assert!(PetAge::parse_string("1").is_ok());
        assert!(PetAge::parse_string("30").is_ok());
        assert!(PetAge::parse_string("0").is_err());
        assert!(PetAge::parse_string("31").is_err());
    }

    #[test]
    fn test_pet_age_as_i32() {
        // Test conversion to i32
        let age = PetAge::parse_i32(25).unwrap();
        assert_eq!(age.as_i32(), 25);
    }

    #[test]
    fn test_weight_kg_valid() {
        // Test valid weights within range (1.0-150.0)
        assert!(WeightKg::parse_f32(1.0).is_ok());
        assert!(WeightKg::parse_f32(50.5).is_ok());
        assert!(WeightKg::parse_f32(150.0).is_ok());
    }

    #[test]
    fn test_weight_kg_invalid() {
        // Test invalid weights outside range
        assert!(WeightKg::parse_f32(0.0).is_err());
        assert!(WeightKg::parse_f32(-1.0).is_err());
        assert!(WeightKg::parse_f32(150.1).is_err());
        assert!(WeightKg::parse_f32(200.0).is_err());
    }

    #[test]
    fn test_weight_kg_nan() {
        // Test NaN rejection
        assert!(WeightKg::parse_f32(f32::NAN).is_err());
    }

    #[test]
    fn test_weight_kg_as_f32() {
        // Test conversion to f32
        let weight = WeightKg::parse_f32(75.5).unwrap();
        assert_eq!(weight.as_f32(), 75.5);
    }
}
