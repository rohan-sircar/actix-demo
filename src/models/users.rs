use serde::{Deserialize, Serialize};

use crate::schema::users;
use crate::utils::regexs;
use chrono;
use validator::Validate;
use validator_derive::*;
use yarte::Template;

#[derive(Debug, Clone, Serialize, Queryable, Identifiable, Deserialize, PartialEq)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Insertable, Deserialize, Validate)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(regex = "regexs::USERNAME_REG", length(min = 4, max = 10))]
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct UserDTO {
    pub name: String,
    pub registration_date: chrono::NaiveDateTime,
}

#[derive(Template)]
#[template(path = "hello.hbs")]
pub struct CardTemplate<'a> {
    pub title: &'a str,
    pub body: String,
    pub num: u32,
}
