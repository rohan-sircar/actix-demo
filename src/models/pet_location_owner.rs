use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{models::pet_basic_info::*, schema::pet_location_owner};

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Selectable,
)]
#[diesel(table_name = pet_location_owner)]
pub struct PetLocationOwner {
    pub id: i32,
    pub pet_basic_info_id: PetBasicInfoId,
    pub owner_name: String,
    pub location: String,
    pub address: Option<String>,
    pub lat: Option<BigDecimal>,
    pub lng: Option<BigDecimal>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_location_owner)]
pub struct NewPetLocationOwner {
    pub pet_basic_info_id: PetBasicInfoId,
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
