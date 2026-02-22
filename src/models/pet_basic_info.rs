use derive_more::Display;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validators::Validator;

use crate::models::pet_enums::*;

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
pub struct PetBasicInfoId(i32);

impl PetBasicInfoId {
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

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Selectable, Identifiable,
)]
#[diesel(table_name = pet_basic_info)]
pub struct PetBasicInfo {
    pub id: PetBasicInfoId,
    pub user_id: UserId,
    pub pet_name: Petname,
    pub pet_type: PetType,
    pub breed: String,
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
    pub breed: String,
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
    pub pet_name: Option<String>,
    pub pet_type: Option<PetType>,
    pub breed: Option<String>,
    pub age: Option<i32>,
    pub weight_kg: Option<f32>,
    pub gender: Option<GenderType>,
    pub size: Option<Option<SizeType>>,
    pub color: Option<Option<String>>,
    pub coat_type: Option<Option<CoatType>>,
}
