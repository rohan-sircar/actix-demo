use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;
use crate::models::pet_enums::*;
use crate::models::pets::{Breedname, NewPetBasicInfo, Petname};
use crate::models::users::UserId;

#[derive(Debug, Clone, Deserialize)]
pub struct PetProfileInsertData {
    // Basic pet information
    pub user_id: UserId,
    pub pet_name: validators::Result<Petname, validators::errors::RegexError>,
    pub pet_type: PetType,
    pub breed: validators::Result<Breedname, validators::errors::RegexError>,
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
    pub fn to_new_pet_basic_info(
        &self,
    ) -> Result<NewPetBasicInfo, DomainError> {
        let mut errors = Vec::new();

        // Validate pet_name
        if let Err(err) = self.pet_name.as_std_result() {
            errors.push(format!(
                "Invalid pet name: {} Must be Alphanumeric and beteen 5-35 characters",
                err
            ));
        }

        // Validate breed
        if let Err(err) = self.breed.as_std_result() {
            errors.push(format!(
                "Invalid breed: {} Must be Alphanumeric and beteen 5-35 characters",
                err
            ));
        }

        // If we have any validation errors, return them all at once
        if !errors.is_empty() {
            let error_message = errors.join("; ");
            return Err(DomainError::new_bad_input_error(error_message));
        }

        // All validations passed, construct the NewPetBasicInfo struct
        let pet_name = self.pet_name.as_std_result().clone().unwrap();
        let breed = self.breed.as_std_result().clone().unwrap();

        Ok(NewPetBasicInfo {
            user_id: self.user_id.clone(),
            pet_name: pet_name.clone(),
            pet_type: self.pet_type.clone(),
            breed: breed.clone(), // Extract the inner String from Breedname
            age: self.age,
            weight_kg: self.weight_kg,
            gender: self.gender.clone(),
            size: self.size.clone(),
            color: self.color.clone(),
            coat_type: self.coat_type.clone(),
        })
    }

    pub fn to_new_pet_personality_traits(
        &self,
        pet_id: &crate::models::pets::PetProfileId,
    ) -> crate::models::pets::NewPetPersonalityTraits {
        crate::models::pets::NewPetPersonalityTraits {
            pet_profile_id: pet_id.clone(),
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
        pet_id: &crate::models::pets::PetProfileId,
    ) -> crate::models::pets::NewPetActivities {
        crate::models::pets::NewPetActivities {
            pet_profile_id: pet_id.clone(),
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
        pet_id: &crate::models::pets::PetProfileId,
    ) -> crate::models::pets::NewPetLocationOwner {
        crate::models::pets::NewPetLocationOwner {
            pet_profile_id: pet_id.clone(),
            owner_name: self.owner_name.clone(),
            location: self.location.clone(),
            address: self.address.clone(),
            lat: self.lat.clone(),
            lng: self.lng.clone(),
        }
    }

    pub fn to_new_pet_adoption_details(
        &self,
        pet_id: &crate::models::pets::PetProfileId,
    ) -> crate::models::pets::NewPetAdoptionDetails {
        crate::models::pets::NewPetAdoptionDetails {
            pet_profile_id: pet_id.clone(),
            special_needs: self.special_needs,
            special_needs_description: self.special_needs_description.clone(),
            adoption_status: self.adoption_status.clone(),
            shelter_name: self.shelter_name.clone(),
        }
    }

    pub fn to_new_pet_profile_images(
        &self,
        pet_id: &crate::models::pets::PetProfileId,
    ) -> Vec<crate::models::pet_profile_images::NewPetProfileImage> {
        self.images
            .iter()
            .map(|image| {
                crate::models::pet_profile_images::NewPetProfileImage {
                    pet_profile_id: pet_id.clone(),
                    image_url: image.image_url.clone(),
                    is_primary: image.is_primary,
                    sort_order: image.sort_order,
                }
            })
            .collect()
    }
}
