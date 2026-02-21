use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use crate::models::pet_activities::UpdatePetActivities;
use crate::models::pet_adoption_details::{AdoptionStatusType, UpdatePetAdoptionDetails};
use crate::models::pet_basic_info::UpdatePetBasicInfo;
use crate::models::pet_enums::*;
use crate::models::pet_location_owner::UpdatePetLocationOwner;
use crate::models::pet_personality_traits::UpdatePetPersonalityTraits;
use crate::models::pet_profile_images::NewPetProfileImage;
use crate::models::users::UserId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PetProfileUpdateData {
    // Basic pet information
    pub user_id: Option<UserId>,
    pub pet_name: Option<String>,
    pub pet_type: Option<PetType>,
    pub breed: Option<String>,
    pub age: Option<i32>,
    pub weight_kg: Option<f32>,
    pub gender: Option<GenderType>,
    pub size: Option<Option<SizeType>>,
    pub color: Option<Option<String>>,
    pub coat_type: Option<Option<CoatType>>,
    
    // Personality traits
    pub bio: Option<Option<String>>,
    pub personality_traits: Option<Option<Vec<Option<String>>>>,
    pub good_with_dogs: Option<Option<bool>>,
    pub good_with_cats: Option<Option<bool>>,
    pub good_with_kids: Option<Option<bool>>,
    pub house_trained: Option<Option<bool>>,
    pub vaccinated: Option<Option<bool>>,
    pub spayed_neutered: Option<Option<bool>>,
    pub microchipped: Option<Option<bool>>,
    
    // Activities
    pub favorite_activities: Option<Option<Vec<String>>>,
    pub likes: Option<Option<Vec<String>>>,
    pub dislikes: Option<Option<Vec<String>>>,
    pub energy_level: Option<Option<EnergyLevelType>>,
    pub trainability: Option<Option<TrainabilityType>>,
    pub barking_level: Option<Option<BarkingLevelType>>,
    
    // Location and owner info
    pub owner_name: Option<String>,
    pub location: Option<String>,
    pub address: Option<Option<String>>,
    pub lat: Option<Option<BigDecimal>>,
    pub lng: Option<Option<BigDecimal>>,
    
    // Adoption details
    pub special_needs: Option<bool>,
    pub special_needs_description: Option<Option<String>>,
    pub adoption_status: Option<Option<AdoptionStatusType>>,
    pub shelter_name: Option<Option<String>>,
    
    // Images - this will be a vec of new images to add, not replace all
    pub images: Vec<NewPetProfileImage>,
}

impl PetProfileUpdateData {
    pub fn to_update_pet_basic_info(&self) -> UpdatePetBasicInfo {
        UpdatePetBasicInfo {
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
    
    pub fn to_update_pet_personality_traits(&self) -> UpdatePetPersonalityTraits {
        UpdatePetPersonalityTraits {
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
    
    pub fn to_update_pet_activities(&self) -> UpdatePetActivities {
        UpdatePetActivities {
            favorite_activities: self.favorite_activities.clone(),
            likes: self.likes.clone(),
            dislikes: self.dislikes.clone(),
            energy_level: self.energy_level.clone(),
            trainability: self.trainability.clone(),
            barking_level: self.barking_level.clone(),
        }
    }
    
    pub fn to_update_pet_location_owner(&self) -> UpdatePetLocationOwner {
        UpdatePetLocationOwner {
            owner_name: self.owner_name.clone(),
            location: self.location.clone(),
            address: self.address.clone(),
            lat: self.lat.clone(),
            lng: self.lng.clone(),
        }
    }
    
    pub fn to_update_pet_adoption_details(&self) -> UpdatePetAdoptionDetails {
        UpdatePetAdoptionDetails {
            special_needs: self.special_needs,
            special_needs_description: self.special_needs_description.clone(),
            adoption_status: self.adoption_status.clone(),
            shelter_name: self.shelter_name.clone(),
        }
    }
}