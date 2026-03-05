use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::EnergyLevelType"]
pub enum EnergyLevelType {
    Low,
    Medium,
    High,
}

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::TrainabilityType"]
pub enum TrainabilityType {
    Easy,
    Moderate,
    Difficult,
}

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::BarkingLevelType"]
pub enum BarkingLevelType {
    Quiet,
    Moderate,
    Frequent,
}

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::PetType"]
pub enum PetType {
    Dog,
    Cat,
}

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::GenderType"]
pub enum GenderType {
    Male,
    Female,
}

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::SizeType"]
pub enum SizeType {
    Toy,
    Small,
    Medium,
    Large,
    Giant,
}

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::CoatType"]
pub enum CoatType {
    Short,
    Medium,
    Long,
    Curly,
    Wire,
    Hairless,
}

#[derive(
    DbEnum, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::AdoptionStatusType"]
pub enum AdoptionStatusType {
    Adoptable,
    Foster,
    Available,
}
