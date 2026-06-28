use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema::likes;
use derive_more::{Display, Into};
use diesel_derive_enum::DbEnum;
use std::str::FromStr;

use super::pets::{PetId, PetName, PetSpecies, PetUuid, PublicPetOwner};
use super::users::DisplayName;
use super::users::UserId;

/// Like direction enum backed by PostgreSQL enum type
#[derive(
    DbEnum, Debug, Clone, Deserialize, Serialize, PartialEq, Eq, ToSchema,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::LikeDirection"]
pub enum LikeDirection {
    Like,
    Dislike,
}

impl std::fmt::Display for LikeDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LikeDirection::Like => write!(f, "like"),
            LikeDirection::Dislike => write!(f, "dislike"),
        }
    }
}

impl FromStr for LikeDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "like" => Ok(LikeDirection::Like),
            "dislike" => Ok(LikeDirection::Dislike),
            _ => Err(format!(
                "invalid direction '{}', expected one of: like, dislike",
                s
            )),
        }
    }
}

/// Newtype for like ID (positive int values)
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
pub struct LikeId(i32);

impl LikeId {
    pub fn as_uint(&self) -> u32 {
        self.0.try_into().unwrap()
    }

    pub fn as_int(&self) -> i32 {
        self.0
    }
}

impl From<LikeId> for u32 {
    fn from(s: LikeId) -> u32 {
        s.0.try_into().unwrap()
    }
}

impl FromStr for LikeId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u32>() {
            num.try_into()
                .map_err(|err| {
                    format!("negative values are not allowed: {}", err)
                })
                .map(LikeId)
        } else {
            Err("expected unsigned int, received string".to_owned())
        }
    }
}

impl TryFrom<u32> for LikeId {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map_err(|err| format!("error while converting like_id: {}", err))
            .map(LikeId)
    }
}

/// Request model for creating a like
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateLike {
    pub pet_uuid: PetUuid,
    pub direction: LikeDirection,
}

/// Queryable model for a like record
#[derive(Debug, Clone, Queryable, Serialize, ToSchema)]
#[diesel(table_name = likes)]
pub struct Like {
    pub id: LikeId,
    pub user_id: UserId,
    pub pet_owner_id: UserId,
    pub pet_id: PetId,
    pub direction: LikeDirection,
    pub created_at: chrono::NaiveDateTime,
}

/// Partial insert model for likes (skip auto-generated id and created_at)
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = likes)]
pub struct NewLike {
    pub user_id: i32,
    pub pet_owner_id: i32,
    pub pet_id: i32,
    pub direction: LikeDirection,
}

impl NewLike {
    pub fn new(
        user_id: UserId,
        pet_owner_id: UserId,
        pet_id: PetId,
        direction: LikeDirection,
    ) -> Self {
        Self {
            user_id: user_id.as_uint() as i32,
            pet_owner_id: pet_owner_id.as_uint() as i32,
            pet_id: pet_id.as_uint() as i32,
            direction,
        }
    }
}

/// Response model for a like (public view)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LikeResponse {
    pub id: LikeId,
    pub pet_uuid: PetUuid,
    pub direction: LikeDirection,
    pub created_at: chrono::NaiveDateTime,
}

impl From<(&Like, &PetUuid)> for LikeResponse {
    fn from((like, pet_uuid): (&Like, &PetUuid)) -> Self {
        Self {
            id: like.id,
            pet_uuid: *pet_uuid,
            direction: like.direction.clone(),
            created_at: like.created_at,
        }
    }
}

/// Response model for a like with full pet and owner data
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LikeWithPet {
    pub pet_uuid: PetUuid,
    pub pet_name: PetName,
    pub species: PetSpecies,
    pub primary_image_uuid: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub liker: Option<PublicPetOwner>,
}

/// Response model for checking if user has already interacted with a pet
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PetInteractionResponse {
    pub interacted: bool,
    pub direction: Option<LikeDirection>,
}

/// Minimal pet info for match display
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MatchPetInfo {
    pub pet_uuid: PetUuid,
    pub pet_name: PetName,
    pub species: PetSpecies,
    pub primary_image_uuid: Option<String>,
}

/// Response model for a mutual match with both pets involved
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MatchWithPets {
    /// The pet the current user liked (other person's pet)
    pub liked_pet: MatchPetInfo,
    /// The pet the other person liked (current user's pet)
    pub liked_by_other_pet: MatchPetInfo,
    /// Display name of the other user
    pub other_owner_name: Option<DisplayName>,
    /// Avatar URL of the other user
    pub other_owner_avatar_url: Option<String>,
    /// UUID of the other user
    pub other_user_uuid: crate::models::users::UserUuid,
    pub is_match: bool,
    pub matched_at: Option<chrono::NaiveDateTime>,
}

/// Queryable model for a match record
#[derive(Debug, Clone, Queryable, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::matches)]
pub struct Match {
    pub id: LikeId,
    pub like_id_a: LikeId,
    pub like_id_b: LikeId,
    pub matched_at: chrono::NaiveDateTime,
}

/// Insertable model for creating a match
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::matches)]
pub struct NewMatch {
    pub like_id_a: i32,
    pub like_id_b: i32,
    pub matched_at: chrono::NaiveDateTime,
}

impl NewMatch {
    pub fn new(like_id_a: LikeId, like_id_b: LikeId) -> Self {
        use chrono::Utc;
        let ts = Utc::now().naive_utc();
        Self {
            like_id_a: like_id_a.as_int(),
            like_id_b: like_id_b.as_int(),
            matched_at: ts,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn like_direction_from_str_valid() {
        assert_eq!(
            LikeDirection::from_str("like").unwrap(),
            LikeDirection::Like
        );
        assert_eq!(
            LikeDirection::from_str("dislike").unwrap(),
            LikeDirection::Dislike
        );
    }

    #[test]
    fn like_direction_from_str_invalid() {
        assert!(LikeDirection::from_str("maybe").is_err());
    }

    #[test]
    fn like_direction_display() {
        assert_eq!(format!("{}", LikeDirection::Like), "like");
        assert_eq!(format!("{}", LikeDirection::Dislike), "dislike");
    }

    #[test]
    fn create_like_deserializes_valid() {
        let json = r#"{"pet_uuid": "550e8400-e29b-41d4-a716-446655440000", "direction": "like"}"#;
        let like = serde_json::from_str::<CreateLike>(json);
        assert!(like.is_ok());
    }

    #[test]
    fn create_like_deserializes_dislike() {
        let json = r#"{"pet_uuid": "550e8400-e29b-41d4-a716-446655440000", "direction": "dislike"}"#;
        let like = serde_json::from_str::<CreateLike>(json);
        assert!(like.is_ok());
        assert_eq!(like.unwrap().direction, LikeDirection::Dislike);
    }

    #[test]
    fn create_like_rejects_invalid_direction() {
        let json = r#"{"pet_uuid": "550e8400-e29b-41d4-a716-446655440000", "direction": "maybe"}"#;
        let like = serde_json::from_str::<CreateLike>(json);
        assert!(like.is_err());
    }

    #[test]
    fn new_like_constructs_correctly() {
        let user_id = UserId::try_from(1u32).unwrap();
        let pet_owner_id = UserId::try_from(2u32).unwrap();
        let pet_id = PetId::try_from(3u32).unwrap();
        let new_like =
            NewLike::new(user_id, pet_owner_id, pet_id, LikeDirection::Like);
        assert_eq!(new_like.user_id, 1);
        assert_eq!(new_like.pet_owner_id, 2);
        assert_eq!(new_like.pet_id, 3);
    }

    #[test]
    fn like_response_from_like_and_uuid() {
        let like = Like {
            id: LikeId::try_from(1u32).unwrap(),
            user_id: UserId::try_from(1u32).unwrap(),
            pet_owner_id: UserId::try_from(2u32).unwrap(),
            pet_id: PetId::try_from(3u32).unwrap(),
            direction: LikeDirection::Like,
            created_at: chrono::NaiveDateTime::default(),
        };
        let pet_uuid = PetUuid::try_from(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
        )
        .unwrap();
        let response = LikeResponse::from((&like, &pet_uuid));
        assert_eq!(response.id.as_uint(), 1);
        assert_eq!(response.direction, LikeDirection::Like);
    }
}
