use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    models::pet_basic_info::PetBasicInfoId, schema::pet_profile_images,
};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct PetProfileImage {
    pub id: i32,
    pub pet_profile_id: PetBasicInfoId,
    pub image_url: String,
    pub is_primary: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_profile_images)]
pub struct NewPetProfileImage {
    pub pet_basic_info_id: PetBasicInfoId,
    pub image_url: String,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}
