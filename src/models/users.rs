use serde::{Deserialize, Serialize};

use crate::schema::users;
use crate::utils::regex;
use derive_more::{Display, Into};
use std::convert::TryFrom;
use std::fmt;
use std::{convert::TryInto, str::FromStr};
use validators::prelude::*;

use super::roles::RoleEnum;

///newtype to constrain id to positive int values
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
)]
#[serde(try_from = "u32", into = "u32")]
pub struct UserId(i32);
impl UserId {
    pub fn as_uint(&self) -> u32 {
        self.0.try_into().unwrap()
    }
}
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
#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq)]
#[validator(regex(regex(regex::USERNAME_REG)))]
pub struct Username(String);
impl Username {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Validator, Clone, DieselNewType)]
#[validator(line(char_length(max = 200)))]
pub struct Password(String);

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Password").field(&"**********").finish()
    }
}

impl Password {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub created_at: chrono::NaiveDateTime,
    // pub role: Vec<RoleEnum>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserWithRoles {
    pub id: UserId,
    pub username: Username,
    pub created_at: chrono::NaiveDateTime,
    pub roles: Vec<RoleEnum>,
}

impl UserWithRoles {
    pub fn from_user(user: &User, roles: &[RoleEnum]) -> UserWithRoles {
        UserWithRoles {
            id: user.id,
            username: user.username.clone(),
            created_at: user.created_at,
            roles: roles.to_vec(),
        }
    }
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: Username,
    #[serde(skip_serializing)]
    pub password: Password,
}

#[derive(Debug, Clone, Deserialize, Queryable)]
pub struct UserLogin {
    pub username: Username,
    #[serde(skip_serializing)]
    pub password: Password,
}

#[derive(Debug, Clone, Deserialize, Queryable)]
pub struct UserAuthDetails {
    pub id: UserId,
    pub username: Username,
    #[serde(skip_serializing)]
    pub password: Password,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserAuthDetailsWithRoles {
    pub id: UserId,
    pub username: Username,
    #[serde(skip_serializing)]
    pub password: Password,
    pub roles: Vec<RoleEnum>,
}

impl UserAuthDetailsWithRoles {
    pub fn from_user(
        user: UserAuthDetails,
        roles: Vec<RoleEnum>,
    ) -> UserAuthDetailsWithRoles {
        UserAuthDetailsWithRoles {
            id: user.id,
            username: user.username,
            password: user.password,
            roles,
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn user_model_refinement_test() {
        //yes I had been watching a lot of star wars lately
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"username":"chewbacca","password":"aeqfq3fq", "role":"role_user", "created_at":"2021-05-12T12:37:56"}"#,
        );
        // println!("{:?}", mb_user);
        assert!(mb_user.is_ok());
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"username":"chew-bacca","password":"aeqfq3fq","role":"role_user","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert!(mb_user.is_ok());
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"username":"chew.bacca","password":"aeqfq3fq","role":"role_user","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert!(mb_user.is_err());
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":-1,"username":"chewbacca","password":"aeqfq3fq","role":"role_user","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert!(mb_user.is_err());
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"username":"ch","password":"aeqfq3fq","role":"role_user","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert!(mb_user.is_err());
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"username":"chaegw;eaef","password":"aeqfq3fq","role":"role_user","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert!(mb_user.is_err());
        let mb_user = serde_json::from_str::<User>(
            r#"{"id":1,"username":"chaegw_eaef","password":"aeqfq3fq","role":"role_user","created_at":"2021-05-12T12:37:56"}"#,
        );
        assert!(mb_user.is_err());
    }

    #[test]
    fn user_model_conversion_test() {
        let user = serde_json::from_str::<User>(
            r#"{"id":1,"username":"chewbacca","password":"aeqfq3fq", "role":"role_user", "created_at":"2021-05-12T12:37:56"}"#,
        ).unwrap();
        let roles = vec![RoleEnum::RoleUser];
        let ur = UserWithRoles::from_user(&user, &roles);
        assert_eq!(ur.id.0 as u32, 1);
    }
}
