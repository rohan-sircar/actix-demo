use diesel::{pg::Pg, prelude::*};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::{models::pet_basic_info::*, schema::pet_adoption_details};

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
    pub pet_basic_info_id: PetBasicInfoId,
    pub special_needs: Option<bool>,
    pub special_needs_description: Option<String>,
    pub adoption_status: Option<AdoptionStatusType>,
    pub shelter_name: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_adoption_details)]
pub struct NewPetAdoptionDetails {
    pub pet_basic_info_id: PetBasicInfoId,
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
