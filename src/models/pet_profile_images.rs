use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{models::pets::PetProfileUuid, schema::pet_profile_images};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Selectable)]
#[diesel(check_for_backend(Pg))]
pub struct PetProfileImage {
    pub id: i32,
    pub pet_profile_uuid: PetProfileUuid,
    pub image_url: String,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = pet_profile_images)]
pub struct NewPetProfileImage {
    pub pet_profile_uuid: PetProfileUuid,
    pub image_url: String,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}

pub struct UpdatePetProfileImage {
    pub image_url: Option<String>,
    pub is_primary: Option<Option<bool>>,
}

/// Request DTO for adding a single image to a pet profile
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddImageRequest {
    pub image_url: String,
    pub is_primary: Option<bool>,
}

/// Request DTO for adding multiple images to a pet profile
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddImagesRequest {
    pub image_urls: Vec<String>,
}
