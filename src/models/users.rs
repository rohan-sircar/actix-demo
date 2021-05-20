use serde::{Deserialize, Serialize};

use crate::schema::users;
use crate::utils::regex;
use derive_more::{Display, Into};
use std::convert::TryFrom;
use std::{convert::TryInto, str::FromStr};
use validators::prelude::*;

///newtype to constrain id to positive int values
///
///sqlite does not allow u32 for primary keys
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
pub struct UserId(i32);
impl From<UserId> for u32 {
    fn from(s: UserId) -> u32 {
        //this should be safe to unwrap since our newtype
        //does not allow negative values
        s.0.try_into().unwrap()
    }
}

impl FromStr for UserId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u32>() {
            num.try_into()
                .map_err(|err| {
                    format!("negative values are not allowed: {}", err)
                })
                .map(UserId)
        } else {
            Err("expected unsigned int, received string".to_owned())
        }
    }
}

impl TryFrom<u32> for UserId {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map_err(|err| format!("error while converting user_id: {}", err))
            .map(UserId)
    }
}
#[derive(Validator, Debug, Clone, DieselNewType)]
#[validator(regex(regex::USERNAME_REG))]
pub struct Username(String);
impl Username {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Validator, Debug, Clone, DieselNewType)]
#[validator(line(char_length(max = 200)))]
pub struct Password(String);

impl Password {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "users"]
pub struct User {
    pub id: UserId,
    pub name: Username,
    #[serde(skip_serializing)]
    pub password: Password,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[table_name = "users"]
pub struct NewUser {
    pub name: Username,
    #[serde(skip_serializing)]
    pub password: Password,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "u16")]
pub struct PaginationOffset(u16);
impl PaginationOffset {
    pub fn as_uint(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PaginationOffset {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value <= 2500 {
            Ok(PaginationOffset(value))
        } else {
            Err("Failed to validate".to_owned())
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "u16")]
pub struct PaginationLimit(u16);
impl PaginationLimit {
    pub fn as_uint(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PaginationLimit {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value <= 50 {
            Ok(PaginationLimit(value))
        } else {
            Err("Failed to validate".to_owned())
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "u16")]
pub struct PaginationPage(u16);
impl PaginationPage {
    pub fn as_uint(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PaginationPage {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value <= 50 {
            Ok(PaginationPage(value))
        } else {
            Err("Failed to validate".to_owned())
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    pub page: PaginationPage,
    pub limit: PaginationLimit,
}

impl Pagination {
    pub fn calc_offset(&self) -> PaginationOffset {
        let res = self.page.as_uint() * self.limit.as_uint();
        PaginationOffset::try_from(res).unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQueryString(String);

impl SearchQueryString {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub q: SearchQueryString,
    // pub pagination: Pagination
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn user_model_refinement_test() {
        //yes I had been watching a lot of star wars lately
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"name":"chewbacca","password":"aeqfq3fq","created_at":"2021-05-12T12:37:56"}"#,
        );
        // println!("{:?}", mb_user);
        assert_eq!(mb_user.is_ok(), true);
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"name":"chew-bacca","password":"aeqfq3fq","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert_eq!(mb_user.is_ok(), true);
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"name":"chew.bacca","password":"aeqfq3fq","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert_eq!(mb_user.is_ok(), false);
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":-1,"name":"chewbacca","password":"aeqfq3fq","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert_eq!(mb_user.is_ok(), false);
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"name":"ch","password":"aeqfq3fq","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert_eq!(mb_user.is_ok(), false);
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"name":"chaegw;eaef","password":"aeqfq3fq","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert_eq!(mb_user.is_ok(), false);
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"name":"chaegw_eaef","password":"aeqfq3fq","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert_eq!(mb_user.is_ok(), false);
    }

    #[test]
    fn pagination_refinement_test() {
        let mb_pag =
            serde_json::from_str::<Pagination>(r#"{"limit":5,"page":5}"#);
        // println!("{:?}", mb_pag);
        assert_eq!(mb_pag.is_ok(), true);
        let mb_pag =
            serde_json::from_str::<Pagination>(r#"{"limit":51,"page":5}"#);
        assert_eq!(mb_pag.is_ok(), false);
        let mb_pag =
            serde_json::from_str::<Pagination>(r#"{"limit":5,"page":51}"#);
        assert_eq!(mb_pag.is_ok(), false);
    }
}
