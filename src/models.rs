use serde::{Deserialize, Serialize};

use crate::schema::users;
use yarte::Template;

#[derive(Debug, Clone, Serialize, Queryable, Identifiable, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Insertable, Deserialize)]
#[table_name = "users"]
pub struct NewUser {
    pub name: String,
}

#[derive(Template)]
#[template(path = "hello.hbs")]
pub struct CardTemplate<'a> {
    pub title: &'a str,
    pub body: String,
    pub num: u32,
}
