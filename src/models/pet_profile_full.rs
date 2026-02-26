use serde::{Deserialize, Serialize};

use crate::models::pet_profile_images::PetProfileImage;
use crate::models::pets::PetActivities;
use crate::models::pets::PetAdoptionDetails;
use crate::models::pets::PetBasicInfo;
use crate::models::pets::PetLocationOwner;
use crate::models::pets::PetPersonalityTraits;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FullPetProfile {
    pub basic_info: PetBasicInfo,
    pub personality_traits: Option<PetPersonalityTraits>,
    pub activities: Option<PetActivities>,
    pub location_owner: Option<PetLocationOwner>,
    pub adoption_details: Option<PetAdoptionDetails>,
    pub images: Vec<PetProfileImage>,
}
