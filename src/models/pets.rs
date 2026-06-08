use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validators::prelude::*;

use crate::schema::pet_personality_traits;
use derive_more::{Display, Into};
use diesel_derive_enum::DbEnum;
use std::str::FromStr;

/// Newtype for pet ID (positive int values)
#[derive(
    Debug,
    Clone,
    Eq,
    Hash,
    PartialEq,
    Deserialize,
    Display,
    Into,
    Serialize,
    DieselNewType,
    Copy,
    ToSchema,
)]
#[serde(try_from = "u32", into = "u32")]
pub struct PetId(i32);

impl PetId {
    pub fn as_uint(&self) -> u32 {
        self.0.try_into().unwrap()
    }

    pub fn as_int(&self) -> i32 {
        self.0
    }
}

impl From<PetId> for u32 {
    fn from(s: PetId) -> u32 {
        s.0.try_into().unwrap()
    }
}

impl FromStr for PetId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u32>() {
            num.try_into()
                .map_err(|err| {
                    format!("negative values are not allowed: {}", err)
                })
                .map(PetId)
        } else {
            Err("expected unsigned int, received string".to_owned())
        }
    }
}

impl TryFrom<u32> for PetId {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map_err(|err| format!("error while converting pet_id: {}", err))
            .map(PetId)
    }
}

/// Newtype for pet UUID (public-facing identifier)
#[derive(
    Debug,
    Clone,
    Eq,
    Hash,
    PartialEq,
    Deserialize,
    Display,
    Into,
    Serialize,
    DieselNewType,
    Copy,
    ToSchema,
)]
#[serde(try_from = "String", into = "String")]
pub struct PetUuid(Uuid);

impl PetUuid {
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<PetUuid> for String {
    fn from(s: PetUuid) -> String {
        s.0.to_string()
    }
}

impl FromStr for PetUuid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s)
            .map(PetUuid)
            .map_err(|e| format!("invalid UUID format: {}", e))
    }
}

impl TryFrom<String> for PetUuid {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse::<PetUuid>()
    }
}

/// Newtype for trait ID (positive int values)
#[derive(
    Debug,
    Clone,
    Eq,
    Hash,
    PartialEq,
    Deserialize,
    Display,
    Into,
    Serialize,
    DieselNewType,
    Copy,
    ToSchema,
)]
#[serde(try_from = "u32", into = "u32")]
pub struct TraitId(i32);

impl TraitId {
    pub fn as_uint(&self) -> u32 {
        self.0.try_into().unwrap()
    }
}

impl From<TraitId> for u32 {
    fn from(s: TraitId) -> u32 {
        s.0.try_into().unwrap()
    }
}

impl FromStr for TraitId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u32>() {
            num.try_into()
                .map_err(|err| {
                    format!("negative values are not allowed: {}", err)
                })
                .map(TraitId)
        } else {
            Err("expected unsigned int, received string".to_owned())
        }
    }
}

impl TryFrom<u32> for TraitId {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map_err(|err| format!("error while converting trait_id: {}", err))
            .map(TraitId)
    }
}

/// Validator for pet name
#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(min = 2, max = 100)))]
pub struct PetName(String);

impl PetName {
    pub fn new(value: String) -> Result<Self, String> {
        Self::parse_string(&value).map_err(|e| e.to_string())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

/// Validator for species
#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(min = 3, max = 50)))]
pub struct PetSpecies(String);

impl PetSpecies {
    pub fn new(value: String) -> Result<Self, String> {
        Self::parse_string(&value).map_err(|e| e.to_string())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

/// Validator for breed
#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(min = 5, max = 200)))]
pub struct PetBreed(String);

impl PetBreed {
    pub fn new(value: String) -> Result<Self, String> {
        Self::parse_string(&value).map_err(|e| e.to_string())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

/// Pet gender enum backed by PostgreSQL enum type
#[derive(
    DbEnum,
    Debug,
    Clone,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    ToSchema,
    Display,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::PetGender"]
pub enum PetGender {
    Male,
    Female,
    Unspecified,
}

impl FromStr for PetGender {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "male" => Ok(PetGender::Male),
            "female" => Ok(PetGender::Female),
            "unspecified" => Ok(PetGender::Unspecified),
            _ => Err(format!(
                "invalid gender '{}', expected one of: male, female, unspecified",
                s
            )),
        }
    }
}

/// Validator for color/markings
#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(max = 200)))]
pub struct PetColorMarkings(String);

impl PetColorMarkings {
    pub fn new(value: String) -> Result<Self, String> {
        Self::parse_string(&value).map_err(|e| e.to_string())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

/// Validator for description
#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(max = 5000)))]
pub struct PetDescription(String);

impl PetDescription {
    pub fn new(value: String) -> Result<Self, String> {
        Self::parse_string(&value).map_err(|e| e.to_string())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

/// Request model for creating a pet
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePet {
    pub name: PetName,
    pub species: PetSpecies,
    pub breed: Option<PetBreed>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub gender: Option<PetGender>,
    #[serde(default, deserialize_with = "deserialize_weight")]
    pub weight: Option<f64>,
    pub color_markings: Option<PetColorMarkings>,
    pub description: Option<PetDescription>,
    #[serde(default)]
    pub traits: Vec<String>,
}

fn deserialize_weight<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<f64>::deserialize(deserializer)?;
    if let Some(w) = opt {
        if w < 0.0 {
            return Err(serde::de::Error::custom("Weight cannot be negative"));
        }
    }
    Ok(opt)
}

/// Diesel insertable model for pet_personality_traits junction
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_personality_traits)]
pub struct NewPetTrait {
    pub pet_id: i32,
    pub trait_id: i32,
}

/// Queryable model for personality trait
#[derive(Debug, Clone, Queryable, Serialize, ToSchema)]
#[diesel(table_name = personality_traits)]
pub struct PersonalityTrait {
    pub id: TraitId,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
}

/// Response model for a pet trait reference
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PetTrait {
    pub id: TraitId,
    pub name: String,
}

/// Queryable model for a pet with all fields
#[derive(Debug, Clone, Queryable, Serialize, ToSchema)]
#[diesel(table_name = pets)]
pub struct Pet {
    pub id: PetId,
    pub pet_uuid: PetUuid,
    pub user_id: crate::models::users::UserId,
    pub name: PetName,
    pub species: PetSpecies,
    pub breed: Option<PetBreed>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub gender: Option<PetGender>,
    pub weight: Option<f64>,
    pub color_markings: Option<PetColorMarkings>,
    pub description: Option<PetDescription>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

/// Partial update model with custom deserializer for tracking sent fields
#[derive(Debug, Clone, Serialize, PartialEq, ToSchema)]
pub struct UpdatePet {
    pub name: Option<PetName>,
    pub species: Option<PetSpecies>,
    pub breed: Option<Option<PetBreed>>,
    pub date_of_birth: Option<Option<chrono::NaiveDate>>,
    pub gender: Option<Option<PetGender>>,
    pub weight: Option<Option<f64>>,
    pub color_markings: Option<Option<PetColorMarkings>>,
    pub description: Option<Option<PetDescription>>,
    pub traits: Option<Vec<String>>,
    #[serde(skip, default)]
    pub _sent_fields: std::collections::HashSet<String>,
}

impl<'de> Deserialize<'de> for UpdatePet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        let mut name = None;
        let mut species = None;
        let mut breed = None;
        let mut date_of_birth = None;
        let mut gender = None;
        let mut weight = None;
        let mut color_markings = None;
        let mut description = None;
        let mut traits = None;
        let mut sent = std::collections::HashSet::new();

        if let serde_json::Value::Object(map) = value {
            for (key, val) in map {
                sent.insert(key.clone());
                match key.as_str() {
                    "name" => {
                        if val.is_null() {
                            name = None;
                        } else if let Some(s) = val.as_str() {
                            name = Some(
                                PetName::new(s.to_string())
                                    .map_err(serde::de::Error::custom)?,
                            );
                        }
                    }
                    "species" => {
                        if val.is_null() {
                            species = None;
                        } else if let Some(s) = val.as_str() {
                            species = Some(
                                PetSpecies::new(s.to_string())
                                    .map_err(serde::de::Error::custom)?,
                            );
                        }
                    }
                    "breed" => {
                        if val.is_null() {
                            breed = Some(None);
                        } else if let Some(s) = val.as_str() {
                            breed = Some(Some(
                                PetBreed::new(s.to_string())
                                    .map_err(serde::de::Error::custom)?,
                            ));
                        }
                    }
                    "date_of_birth" => {
                        if val.is_null() {
                            date_of_birth = Some(None);
                        } else if let Some(s) = val.as_str() {
                            date_of_birth = Some(
                                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                                    .ok()
                                    .map(Some)
                                    .ok_or_else(|| {
                                        serde::de::Error::custom(
                                            "Invalid date format, expected YYYY-MM-DD",
                                        )
                                    })?,
                            );
                        }
                    }
                    "gender" => {
                        if val.is_null() {
                            gender = Some(None);
                        } else if let Some(s) = val.as_str() {
                            let parsed: Result<PetGender, _> = s.parse();
                            gender = Some(Some(
                                parsed.map_err(serde::de::Error::custom)?,
                            ));
                        }
                    }
                    "weight" => {
                        if val.is_null() {
                            weight = Some(None);
                        } else if let Some(w) = val.as_f64() {
                            if w < 0.0 {
                                return Err(serde::de::Error::custom(
                                    "Weight cannot be negative",
                                ));
                            }
                            weight = Some(Some(w));
                        }
                    }
                    "color_markings" => {
                        if val.is_null() {
                            color_markings = Some(None);
                        } else if let Some(s) = val.as_str() {
                            color_markings = Some(Some(
                                PetColorMarkings::new(s.to_string())
                                    .map_err(serde::de::Error::custom)?,
                            ));
                        }
                    }
                    "description" => {
                        if val.is_null() {
                            description = Some(None);
                        } else if let Some(s) = val.as_str() {
                            description = Some(Some(
                                PetDescription::new(s.to_string())
                                    .map_err(serde::de::Error::custom)?,
                            ));
                        }
                    }
                    "traits" => {
                        if let Some(arr) = val.as_array() {
                            traits = Some(
                                arr.iter()
                                    .filter_map(|v| {
                                        v.as_str().map(String::from)
                                    })
                                    .collect(),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(UpdatePet {
            name,
            species,
            breed,
            date_of_birth,
            gender,
            weight,
            color_markings,
            description,
            traits,
            _sent_fields: sent,
        })
    }
}

impl UpdatePet {
    pub fn should_update(&self, field: &str) -> bool {
        self._sent_fields.contains(field)
    }
}

/// Public-facing pet view (without owner info)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PublicPet {
    pub id: PetId,
    pub pet_uuid: PetUuid,
    pub name: PetName,
    pub species: PetSpecies,
    pub breed: Option<PetBreed>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub gender: Option<PetGender>,
    pub weight: Option<f64>,
    pub color_markings: Option<PetColorMarkings>,
    pub description: Option<PetDescription>,
    pub traits: Vec<PetTrait>,
}

impl From<(&Pet, Vec<PetTrait>)> for PublicPet {
    fn from((pet, traits): (&Pet, Vec<PetTrait>)) -> Self {
        PublicPet {
            id: pet.id,
            pet_uuid: pet.pet_uuid,
            name: pet.name.clone(),
            species: pet.species.clone(),
            breed: pet.breed.clone(),
            date_of_birth: pet.date_of_birth,
            gender: pet.gender.clone(),
            weight: pet.weight,
            color_markings: pet.color_markings.clone(),
            description: pet.description.clone(),
            traits,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_pet_deserializes_valid_fields() {
        let json = r#"{
            "name": "Buddy",
            "species": "dog",
            "breed": "Golden Retriever",
            "date_of_birth": "2020-05-15",
            "gender": "male",
            "weight": 30.5,
            "color_markings": "Golden",
            "description": "A friendly dog",
            "traits": ["playful", "loyal"]
        }"#;

        let pet = serde_json::from_str::<CreatePet>(json);
        assert!(pet.is_ok());

        let pet = pet.unwrap();
        assert_eq!(pet.name.0, "Buddy");
        assert_eq!(pet.species.0, "dog");
        assert!(pet.breed.is_some());
        assert!(pet.date_of_birth.is_some());
        assert!(pet.gender.is_some());
        assert_eq!(pet.weight, Some(30.5));
        assert_eq!(pet.traits, vec!["playful", "loyal"]);
    }

    #[test]
    fn create_pet_accepts_partial_fields() {
        let json = r#"{"name": "Whiskers", "species": "cat"}"#;

        let pet = serde_json::from_str::<CreatePet>(json);
        assert!(pet.is_ok());

        let pet = pet.unwrap();
        assert_eq!(pet.name.0, "Whiskers");
        assert_eq!(pet.species.0, "cat");
        assert!(pet.breed.is_none());
        assert!(pet.traits.is_empty());
    }

    #[test]
    fn create_pet_rejects_negative_weight() {
        let json = r#"{"name": "Buddy", "species": "dog", "weight": -5.0}"#;

        let pet = serde_json::from_str::<CreatePet>(json);
        assert!(pet.is_err());
    }

    #[test]
    fn update_pet_deserializer_tracks_sent_fields() {
        let json = r#"{"name": "Rex", "species": "dog"}"#;
        let update = serde_json::from_str::<UpdatePet>(json);
        assert!(update.is_ok());

        let update = update.unwrap();
        assert!(update.should_update("name"));
        assert!(update.should_update("species"));
        assert!(!update.should_update("breed"));
        assert!(!update.should_update("weight"));
    }

    #[test]
    fn update_pet_deserializer_detects_null_for_clearable_fields() {
        let json = r#"{"breed": null, "gender": null}"#;
        let update = serde_json::from_str::<UpdatePet>(json);
        assert!(update.is_ok());

        let update = update.unwrap();
        assert!(update.should_update("breed"));
        assert_eq!(update.breed, Some(None));
        assert!(update.should_update("gender"));
        assert_eq!(update.gender, Some(None));
    }

    #[test]
    fn update_pet_deserializer_handles_traits_replacement() {
        let json = r#"{"traits": ["calm", "independent"]}"#;
        let update = serde_json::from_str::<UpdatePet>(json);
        assert!(update.is_ok());

        let update = update.unwrap();
        assert!(update.should_update("traits"));
        assert_eq!(
            update.traits,
            Some(vec!["calm".to_string(), "independent".to_string()])
        );
    }

    #[test]
    fn update_pet_rejects_invalid_name() {
        let json = format!(r#"{{"name": "{}"}}"#, "a".repeat(101));
        let update = serde_json::from_str::<UpdatePet>(&json);
        assert!(update.is_err());
    }

    #[test]
    fn update_pet_rejects_negative_weight() {
        let json = r#"{"weight": -10.0}"#;
        let update = serde_json::from_str::<UpdatePet>(json);
        assert!(update.is_err());
    }

    #[test]
    fn update_pet_accepts_date_format() {
        let json = r#"{"date_of_birth": "2019-03-20"}"#;
        let update = serde_json::from_str::<UpdatePet>(json);
        assert!(update.is_ok());
    }

    #[test]
    fn update_pet_rejects_invalid_date_format() {
        let json = r#"{"date_of_birth": "2019/03/20"}"#;
        let update = serde_json::from_str::<UpdatePet>(json);
        assert!(update.is_err());
    }

    #[test]
    fn public_pet_converts_from_pet_and_traits() {
        let pet = Pet {
            id: PetId::try_from(1u32).unwrap(),
            pet_uuid: PetUuid::try_from(
                "550e8400-e29b-41d4-a716-446655440000".to_string(),
            )
            .unwrap(),
            user_id: crate::models::users::UserId::try_from(1u32).unwrap(),
            name: PetName("Buddy".to_string()),
            species: PetSpecies("dog".to_string()),
            breed: Some(PetBreed("Labrador".to_string())),
            date_of_birth: Some(
                chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            ),
            gender: Some(PetGender::Male),
            weight: Some(25.0),
            color_markings: Some(PetColorMarkings("Yellow".to_string())),
            description: Some(PetDescription("Friendly".to_string())),
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        };

        let traits = vec![
            PetTrait {
                id: TraitId::try_from(1u32).unwrap(),
                name: "playful".to_string(),
            },
            PetTrait {
                id: TraitId::try_from(2u32).unwrap(),
                name: "loyal".to_string(),
            },
        ];

        let public = PublicPet::from((&pet, traits));
        assert_eq!(public.id, PetId(1));
        assert_eq!(public.name.inner(), "Buddy");
        assert_eq!(public.species.inner(), "dog");
        assert_eq!(public.breed, Some(PetBreed("Labrador".to_string())));
        assert_eq!(public.traits.len(), 2);
    }

    #[test]
    fn pet_name_validation_accepts_valid_lengths() {
        let name = PetName::new("Bb".to_string());
        assert!(name.is_ok());

        let name = PetName::new("a".repeat(100));
        assert!(name.is_ok());
    }

    #[test]
    fn pet_name_validation_rejects_empty() {
        let name = PetName::new("".to_string());
        assert!(name.is_err());
    }

    #[test]
    fn pet_name_validation_rejects_overlong() {
        let name = PetName::new("a".repeat(101));
        assert!(name.is_err());
    }

    #[test]
    fn species_validation_accepts_valid_lengths() {
        let species = PetSpecies::new("dog".to_string());
        assert!(species.is_ok());
    }

    #[test]
    fn species_validation_rejects_overlong() {
        let species = PetSpecies::new("a".repeat(51));
        assert!(species.is_err());
    }
}
