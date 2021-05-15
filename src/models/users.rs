use serde::{Deserialize, Serialize};

use crate::schema::users;
use crate::utils::regex;
use derive_more::{Display, Into};
use std::convert::TryFrom;
use std::{convert::TryInto, str::FromStr};
use validators::prelude::*;

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
            (num as u32)
                .try_into()
                .map_err(|err| {
                    format!("error while converting user_id: {}", err)
                })
                .map(UserId)
        } else {
            Err("negative values are not allowed".to_owned())
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
}
