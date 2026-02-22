use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use crate::models::pet_basic_info::Petname;
use crate::models::pet_enums::*;
use crate::models::pet_adoption_details::AdoptionStatusType;
use crate::models::users::UserId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PetProfileInsertData {
    // Basic pet information
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
    
    // Personality traits
    pub bio: Option<String>,
    pub personality_traits: Option<Vec<Option<String>>>,
    pub good_with_dogs: Option<bool>,
    pub good_with_cats: Option<bool>,
    pub good_with_kids: Option<bool>,
    pub house_trained: Option<bool>,
    pub vaccinated: Option<bool>,
    pub spayed_neutered: Option<bool>,
    pub microchipped: Option<bool>,
    
    // Activities
    pub favorite_activities: Option<Vec<String>>,
    pub likes: Option<Vec<String>>,
    pub dislikes: Option<Vec<String>>,
    pub energy_level: Option<EnergyLevelType>,
    pub trainability: Option<TrainabilityType>,
    pub barking_level: Option<BarkingLevelType>,
    
    // Location and owner info
    pub owner_name: String,
    pub location: String,
    pub address: Option<String>,
    pub lat: Option<BigDecimal>,
    pub lng: Option<BigDecimal>,
    
    // Adoption details
    pub special_needs: bool,
    pub special_needs_description: Option<String>,
    pub adoption_status: Option<AdoptionStatusType>,
    pub shelter_name: Option<String>,
    
    // Images
    pub images: Vec<PetProfileImageInsert>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PetProfileImageInsert {
    pub image_url: String,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}

impl PetProfileInsertData {
    pub fn to_new_pet_basic_info(&self) -> crate::models::pet_basic_info::NewPetBasicInfo {
        crate::models::pet_basic_info::NewPetBasicInfo {
            user_id: self.user_id.clone(),
            pet_name: self.pet_name.clone(),
            pet_type: self.pet_type.clone(),
            breed: self.breed.clone(),
            age: self.age,
            weight_kg: self.weight_kg,
            gender: self.gender.clone(),
            size: self.size.clone(),
            color: self.color.clone(),
            coat_type: self.coat_type.clone(),
        }
    }
    
    pub fn to_new_pet_personality_traits(
        &self, 
        pet_id: &crate::models::pet_basic_info::PetBasicInfoId
    ) -> crate::models::pet_personality_traits::NewPetPersonalityTraits {
        crate::models::pet_personality_traits::NewPetPersonalityTraits {
            pet_basic_info_id: pet_id.clone(),
            bio: self.bio.clone(),
            personality_traits: self.personality_traits.clone(),
            good_with_dogs: self.good_with_dogs,
            good_with_cats: self.good_with_cats,
            good_with_kids: self.good_with_kids,
            house_trained: self.house_trained,
            vaccinated: self.vaccinated,
            spayed_neutered: self.spayed_neutered,
            microchipped: self.microchipped,
        }
    }
    
    pub fn to_new_pet_activities(
        &self, 
        pet_id: &crate::models::pet_basic_info::PetBasicInfoId
    ) -> crate::models::pet_activities::NewPetActivities {
        crate::models::pet_activities::NewPetActivities {
            pet_basic_info_id: pet_id.clone(),
            favorite_activities: self.favorite_activities.clone(),
            likes: self.likes.clone(),
            dislikes: self.dislikes.clone(),
            energy_level: self.energy_level.clone(),
            trainability: self.trainability.clone(),
            barking_level: self.barking_level.clone(),
        }
    }
    
    pub fn to_new_pet_location_owner(
        &self, 
        pet_id: &crate::models::pet_basic_info::PetBasicInfoId
    ) -> crate::models::pet_location_owner::NewPetLocationOwner {
        crate::models::pet_location_owner::NewPetLocationOwner {
            pet_basic_info_id: pet_id.clone(),
            owner_name: self.owner_name.clone(),
            location: self.location.clone(),
            address: self.address.clone(),
            lat: self.lat.clone(),
            lng: self.lng.clone(),
        }
    }
    
    pub fn to_new_pet_adoption_details(
        &self, 
        pet_id: &crate::models::pet_basic_info::PetBasicInfoId
    ) -> crate::models::pet_adoption_details::NewPetAdoptionDetails {
        crate::models::pet_adoption_details::NewPetAdoptionDetails {
            pet_basic_info_id: pet_id.clone(),
            special_needs: self.special_needs,
            special_needs_description: self.special_needs_description.clone(),
            adoption_status: self.adoption_status.clone(),
            shelter_name: self.shelter_name.clone(),
        }
    }
    
    pub fn to_new_pet_profile_images(
        &self, 
        pet_id: &crate::models::pet_basic_info::PetBasicInfoId
    ) -> Vec<crate::models::pet_profile_images::NewPetProfileImage> {
        self.images
            .iter()
            .map(|image| crate::models::pet_profile_images::NewPetProfileImage {
                pet_basic_info_id: pet_id.clone(),
                image_url: image.image_url.clone(),
                is_primary: image.is_primary,
                sort_order: image.sort_order,
            })
            .collect()
    }
}