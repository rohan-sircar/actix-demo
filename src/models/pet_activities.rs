use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    models::{pet_basic_info::*, pet_enums::*},
    schema::pet_activities,
};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[diesel(table_name = pet_activities)]
pub struct PetActivities {
    pub id: i32,
    pub pet_basic_info_id: PetBasicInfoId,
    pub favorite_activities: Option<Vec<String>>,
    pub likes: Option<Vec<String>>,
    pub dislikes: Option<Vec<String>>,
    pub energy_level: Option<EnergyLevelType>,
    pub trainability: Option<TrainabilityType>,
    pub barking_level: Option<BarkingLevelType>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_activities)]
pub struct NewPetActivities {
    pub pet_basic_info_id: PetBasicInfoId,
    pub favorite_activities: Option<Vec<String>>,
    pub likes: Option<Vec<String>>,
    pub dislikes: Option<Vec<String>>,
    pub energy_level: Option<EnergyLevelType>,
    pub trainability: Option<TrainabilityType>,
    pub barking_level: Option<BarkingLevelType>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = pet_activities)]
pub struct UpdatePetActivities {
    pub favorite_activities: Option<Option<Vec<String>>>,
    pub likes: Option<Option<Vec<String>>>,
    pub dislikes: Option<Option<Vec<String>>>,
    pub energy_level: Option<Option<EnergyLevelType>>,
    pub trainability: Option<Option<TrainabilityType>>,
    pub barking_level: Option<Option<BarkingLevelType>>,
}
