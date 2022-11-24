use std::str::FromStr;

use crate::schema::roles;
use derive_more::{Display, Into};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(DbEnum, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
#[serde(rename_all = "snake_case")]
#[DieselType = "Role_name"]
pub enum RoleEnum {
    RoleSuperUser,
    RoleAdmin,
    RoleUser,
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "roles"]
pub struct Role {
    pub id: RoleId,
    pub name: RoleEnum,
}

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
)]
#[serde(try_from = "u32", into = "u32")]
pub struct RoleId(pub i32);
impl From<RoleId> for u32 {
    fn from(s: RoleId) -> u32 {
        //this should be safe to unwrap since our newtype
        //does not allow negative values
        s.0.try_into().unwrap()
    }
}

impl FromStr for RoleId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u32>() {
            num.try_into()
                .map_err(|err| {
                    format!("negative values are not allowed: {}", err)
                })
                .map(RoleId)
        } else {
            Err("expected unsigned int, received string".to_owned())
        }
    }
}

impl TryFrom<u32> for RoleId {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map_err(|err| format!("error while converting user_id: {}", err))
            .map(RoleId)
    }
}
