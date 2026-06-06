use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schema::profiles;
use crate::schema::users;
use crate::utils::regex;
use derive_more::{Display, Into};
use diesel_derive_enum::DbEnum;
use std::convert::TryFrom;
use std::fmt;
use std::{convert::TryInto, str::FromStr};
use validators::prelude::*;

use super::roles::RoleEnum;

#[derive(
    DbEnum, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[ExistingTypePath = "crate::schema::sql_types::OauthProviderType"]
pub enum OAuthProvider {
    Github,
    Google,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &str {
        match self {
            OAuthProvider::Github => "github",
            OAuthProvider::Google => "google",
        }
    }
}

impl std::str::FromStr for OAuthProvider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "github" => Ok(OAuthProvider::Github),
            "google" => Ok(OAuthProvider::Google),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(regex(regex(regex::EMAIL_REG)))]
pub struct Email(String);

impl Email {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Email {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if regex::EMAIL_REG.is_match(&value) {
            Ok(Email(value))
        } else {
            Err(format!("Invalid email address: {}", value))
        }
    }
}

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
    ToSchema,
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
#[derive(
    Validator,
    Debug,
    Clone,
    DieselNewType,
    PartialEq,
    Eq,
    derive_more::Display,
    ToSchema,
)]
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

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "**********")
    }
}

impl Password {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub mod password_serde {
    use super::Password;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        password: &Password,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&password.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Password, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Password(s))
    }
}

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, ToSchema,
)]
#[diesel(table_name = users)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub created_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Insertable, Deserialize, ToSchema)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: Username,
    #[serde(with = "password_serde")]
    pub password: Password,
    pub email: Email,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, ToSchema)]
#[serde(default)]
pub struct UpdateUserProfile {
    pub username: Option<Username>,
    pub email: Option<Email>,
}

#[derive(Debug, Clone, Deserialize, Queryable, ToSchema)]
pub struct UserLogin {
    pub username: Username,
    #[serde(with = "password_serde")]
    pub password: Password,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Queryable)]
#[diesel(table_name = users)]
pub struct UserAuthDetails {
    pub id: UserId,
    pub username: Username,
    pub password: Password,
    pub email: Email,
    pub oauth_provider: Option<OAuthProvider>,
    pub oauth_uid: Option<String>,
}

#[derive(Debug, Clone, Queryable)]
pub struct OAuthUserLookup {
    pub id: UserId,
    pub username: Username,
    pub email: Email,
    pub password: Password,
    pub oauth_provider: Option<OAuthProvider>,
    pub oauth_uid: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserAuthDetailsWithRoles {
    pub id: UserId,
    pub username: Username,
    #[serde(skip_serializing)]
    pub password: Password,
    pub email: Email,
    pub oauth_provider: Option<OAuthProvider>,
    pub oauth_uid: Option<String>,
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
            email: user.email,
            oauth_provider: user.oauth_provider,
            oauth_uid: user.oauth_uid,
            roles,
        }
    }
}
#[derive(Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema, Serialize)]
pub struct Bio(String);

impl Bio {
    const MAX_CHARS: usize = 500;

    pub fn new(value: String) -> Result<Self, String> {
        if value.chars().count() > Self::MAX_CHARS {
            Err(format!(
                "Bio must be at most {} characters",
                Self::MAX_CHARS
            ))
        } else {
            Ok(Self(value))
        }
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Bio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(max = 100)))]
pub struct DisplayName(String);

#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(max = 200)))]
pub struct Location(String);

#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(max = 500)))]
pub struct WebsiteUrl(String);

#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(max = 100)))]
pub struct SocialGithub(String);

#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(max = 100)))]
pub struct SocialTwitter(String);

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, ToSchema,
)]
#[diesel(table_name = profiles)]
pub struct Profile {
    pub id: i32,
    pub user_id: UserId,
    pub bio: Option<Bio>,
    pub display_name: Option<DisplayName>,
    pub location: Option<Location>,
    pub website_url: Option<WebsiteUrl>,
    pub social_github: Option<SocialGithub>,
    pub social_twitter: Option<SocialTwitter>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

fn dummy_user_id() -> UserId {
    UserId(0)
}

#[derive(Debug, Clone, Insertable, Deserialize, ToSchema)]
#[diesel(table_name = profiles)]
pub struct CreateProfile {
    #[serde(skip, default = "dummy_user_id")]
    pub user_id: UserId,
    pub bio: Option<Bio>,
    pub display_name: Option<DisplayName>,
    pub location: Option<Location>,
    pub website_url: Option<WebsiteUrl>,
    pub social_github: Option<SocialGithub>,
    pub social_twitter: Option<SocialTwitter>,
}

#[derive(Debug, Clone, Serialize, PartialEq, ToSchema)]
pub struct UpdateProfile {
    pub bio: Option<Bio>,
    pub display_name: Option<DisplayName>,
    pub location: Option<Location>,
    pub website_url: Option<WebsiteUrl>,
    pub social_github: Option<SocialGithub>,
    pub social_twitter: Option<SocialTwitter>,
    #[serde(skip, default)]
    pub _sent_fields: std::collections::HashSet<String>,
}

impl<'de> Deserialize<'de> for UpdateProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        let mut bio = None;
        let mut display_name = None;
        let mut location = None;
        let mut website_url = None;
        let mut social_github = None;
        let mut social_twitter = None;
        let mut sent = std::collections::HashSet::new();

        if let serde_json::Value::Object(map) = value {
            for (key, val) in map {
                sent.insert(key.clone());
                match key.as_str() {
                    "bio" => {
                        if val.is_null() {
                            bio = None;
                        } else if let Some(s) = val.as_str() {
                            bio = Some(
                                Bio::new(s.to_string())
                                    .map_err(serde::de::Error::custom)?,
                            );
                        }
                    }
                    "display_name" => {
                        if val.is_null() {
                            display_name = None;
                        } else if let Some(s) = val.as_str() {
                            display_name = Some(DisplayName(s.to_string()));
                        }
                    }
                    "location" => {
                        if val.is_null() {
                            location = None;
                        } else if let Some(s) = val.as_str() {
                            location = Some(Location(s.to_string()));
                        }
                    }
                    "website_url" => {
                        if val.is_null() {
                            website_url = None;
                        } else if let Some(s) = val.as_str() {
                            website_url = Some(WebsiteUrl(s.to_string()));
                        }
                    }
                    "social_github" => {
                        if val.is_null() {
                            social_github = None;
                        } else if let Some(s) = val.as_str() {
                            social_github = Some(SocialGithub(s.to_string()));
                        }
                    }
                    "social_twitter" => {
                        if val.is_null() {
                            social_twitter = None;
                        } else if let Some(s) = val.as_str() {
                            social_twitter = Some(SocialTwitter(s.to_string()));
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(UpdateProfile {
            bio,
            display_name,
            location,
            website_url,
            social_github,
            social_twitter,
            _sent_fields: sent,
        })
    }
}

impl UpdateProfile {
    pub fn should_update(&self, field: &str) -> bool {
        self._sent_fields.contains(field)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PublicProfile {
    pub user_id: UserId,
    pub bio: Option<Bio>,
    pub display_name: Option<DisplayName>,
    pub location: Option<Location>,
    pub website_url: Option<WebsiteUrl>,
    pub social_github: Option<SocialGithub>,
    pub social_twitter: Option<SocialTwitter>,
}

impl From<&Profile> for PublicProfile {
    fn from(profile: &Profile) -> Self {
        Self {
            user_id: profile.user_id,
            bio: profile.bio.clone(),
            display_name: profile.display_name.clone(),
            location: profile.location.clone(),
            website_url: profile.website_url.clone(),
            social_github: profile.social_github.clone(),
            social_twitter: profile.social_twitter.clone(),
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
        assert!(mb_user.is_ok());
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
            r#"{"id":1,"username":"chaegw eaef","password":"aeqfq3fq","role":"role_user","created_at":"2021-05-12T12:37:56"}"#,
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
