use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    models::pets::PetProfileId, schema::pet_profile_images,
};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Selectable)]
#[diesel(check_for_backend(Pg))]
pub struct PetProfileImage {
    pub id: i32,
    pub pet_profile_id: PetProfileId,
    pub image_url: String,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = pet_profile_images)]
pub struct NewPetProfileImage {
    pub pet_profile_id: PetProfileId,
    pub image_url: String,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}
