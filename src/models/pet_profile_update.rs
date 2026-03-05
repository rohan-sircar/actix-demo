use bigdecimal::BigDecimal;
use serde::Deserialize;

use crate::errors::DomainError;
use crate::models::pet_enums::*;
use crate::models::pet_profile_images::NewPetProfileImage;
use crate::models::pets::UpdatePetActivities;
use crate::models::pets::UpdatePetLocationOwner;
use crate::models::pets::UpdatePetPersonalityTraits;
use crate::models::pets::UpdatePetAdoptionDetails;
use crate::models::pets::{Breedname, Petname, UpdatePetBasicInfo};
use crate::models::users::UserId;

#[derive(Debug, Clone, Deserialize)]
pub struct PetBasicInfoUpdate {
    // Basic pet information
    pub user_id: Option<UserId>,
    pub pet_name:
        Option<validators::Result<Petname, validators::errors::RegexError>>,
    pub pet_type: Option<PetType>,
    pub breed:
        Option<validators::Result<Breedname, validators::errors::RegexError>>,
    pub age: Option<i32>,
    pub weight_kg: Option<f32>,
    pub gender: Option<GenderType>,
    pub size: Option<Option<SizeType>>,
    pub color: Option<Option<String>>,
    pub coat_type: Option<Option<CoatType>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PetPersonalityTraitsUpdate {
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct PetActivitiesUpdate {
    // Activities
    pub favorite_activities: Option<Option<Vec<String>>>,
    pub likes: Option<Option<Vec<String>>>,
    pub dislikes: Option<Option<Vec<String>>>,
    pub energy_level: Option<Option<EnergyLevelType>>,
    pub trainability: Option<Option<TrainabilityType>>,
    pub barking_level: Option<Option<BarkingLevelType>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PetLocationOwnerUpdate {
    // Location and owner info
    pub owner_name: Option<String>,
    pub location: Option<String>,
    pub address: Option<Option<String>>,
    pub lat: Option<Option<BigDecimal>>,
    pub lng: Option<Option<BigDecimal>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PetAdoptionDetailsUpdate {
    // Adoption details
    pub special_needs: Option<bool>,
    pub special_needs_description: Option<Option<String>>,
    pub adoption_status: Option<Option<AdoptionStatusType>>,
    pub shelter_name: Option<Option<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PetProfileUpdateData {
    pub basic_info: Option<PetBasicInfoUpdate>,
    pub personality_traits: Option<PetPersonalityTraitsUpdate>,
    pub activities: Option<PetActivitiesUpdate>,
    pub location_owner: Option<PetLocationOwnerUpdate>,
    pub adoption_details: Option<PetAdoptionDetailsUpdate>,
    // Images - this will be a vec of new images to add, not replace all
    pub images: Option<Vec<NewPetProfileImage>>,
}

impl PetProfileUpdateData {
    pub fn to_update_pet_basic_info(
        &self,
    ) -> Result<Option<UpdatePetBasicInfo>, DomainError> {
        let basic_info = match &self.basic_info {
            Some(info) => info,
            None => return Ok(None),
        };

        let mut errors = Vec::new();

        // Validate pet_name
        if let Some(pet_name) = &basic_info.pet_name {
            if let Err(err) = pet_name.as_std_result() {
                errors.push(format!(
                    "Invalid pet name: {} Must be Alphanumeric and beteen 5-35 characters",
                    err
                ));
            }
        }

        // Validate breed
        if let Some(breed) = &basic_info.breed {
            if let Err(err) = breed.as_std_result() {
                errors.push(format!(
                    "Invalid breed: {} Must be Alphanumeric and beteen 5-35 characters",
                    err
                ));
            }
        }

        // If we have any validation errors, return them all at once
        if !errors.is_empty() {
            let error_message = errors.join("; ");
            return Err(DomainError::new_bad_input_error(error_message));
        }

        // All validations passed, construct the UpdatePetBasicInfo struct
        Ok(Some(UpdatePetBasicInfo {
            pet_name: basic_info
                .pet_name
                .as_ref()
                .map(|v| v.as_std_result().clone().unwrap()),
            pet_type: basic_info.pet_type.clone(),
            breed: basic_info
                .breed
                .as_ref()
                .map(|v| v.as_std_result().clone().unwrap()),
            age: basic_info.age,
            weight_kg: basic_info.weight_kg,
            gender: basic_info.gender.clone(),
            size: basic_info.size.clone(),
            color: basic_info.color.clone(),
            coat_type: basic_info.coat_type.clone(),
        }))
    }

    pub fn to_update_pet_personality_traits(
        &self,
    ) -> Option<UpdatePetPersonalityTraits> {
        match &self.personality_traits {
            Some(p) => Some(UpdatePetPersonalityTraits {
                bio: p.bio.clone(),
                personality_traits: p.personality_traits.clone(),
                good_with_dogs: p.good_with_dogs,
                good_with_cats: p.good_with_cats,
                good_with_kids: p.good_with_kids,
                house_trained: p.house_trained,
                vaccinated: p.vaccinated,
                spayed_neutered: p.spayed_neutered,
                microchipped: p.microchipped,
            }),
            None => None,
        }
    }

    pub fn to_update_pet_activities(&self) -> Option<UpdatePetActivities> {
        match &self.activities {
            Some(a) => Some(UpdatePetActivities {
                favorite_activities: a.favorite_activities.clone(),
                likes: a.likes.clone(),
                dislikes: a.dislikes.clone(),
                energy_level: a.energy_level.clone(),
                trainability: a.trainability.clone(),
                barking_level: a.barking_level.clone(),
            }),
            None => None,
        }
    }

    pub fn to_update_pet_location_owner(
        &self,
    ) -> Option<UpdatePetLocationOwner> {
        match &self.location_owner {
            Some(l) => Some(UpdatePetLocationOwner {
                owner_name: l.owner_name.clone(),
                location: l.location.clone(),
                address: l.address.clone(),
                lat: l.lat.clone(),
                lng: l.lng.clone(),
            }),
            None => None,
        }
    }

    pub fn to_update_pet_adoption_details(
        &self,
    ) -> Option<UpdatePetAdoptionDetails> {
        match &self.adoption_details {
            Some(a) => Some(UpdatePetAdoptionDetails {
                special_needs: a.special_needs,
                special_needs_description: a.special_needs_description.clone(),
                adoption_status: a.adoption_status.clone(),
                shelter_name: a.shelter_name.clone(),
            }),
            None => None,
        }
    }
}
