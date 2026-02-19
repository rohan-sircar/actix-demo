use serde::{Deserialize, Serialize};

use crate::models::pet_activities::PetActivities;
use crate::models::pet_adoption_details::PetAdoptionDetails;
use crate::models::pet_basic_info::PetBasicInfo;
use crate::models::pet_location_owner::PetLocationOwner;
use crate::models::pet_personality_traits::PetPersonalityTraits;
use crate::models::pet_profile_images::PetProfileImage;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FullPetProfile {
    pub basic_info: PetBasicInfo,
    pub personality_traits: Option<PetPersonalityTraits>,
    pub activities: Option<PetActivities>,
    pub location_owner: Option<PetLocationOwner>,
    pub adoption_details: Option<PetAdoptionDetails>,
    pub images: Vec<PetProfileImage>,
}
