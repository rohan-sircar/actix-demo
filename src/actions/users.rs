use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use crate::errors;
use crate::models;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::rc::Rc;

pub fn find_user_by_uid(
    uid: i32,
    conn: &SqliteConnection,
) -> Result<Option<models::UserDTO>, errors::DomainError> {
    use crate::schema::users::dsl::*;

    let maybe_user = users
        .select((name, created_at))
        .find(uid)
        .first::<models::UserDTO>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn _find_user_by_name(
    user_name: String,
    conn: &SqliteConnection,
) -> Result<Option<models::UserDTO>, errors::DomainError> {
    let maybe_user = _get_user_by_name(user_name)
        .first::<models::UserDTO>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn get_all(
    conn: &SqliteConnection,
) -> Result<Option<Vec<models::UserDTO>>, errors::DomainError> {
    use crate::schema::users::dsl::*;
    Ok(users
        .select((name, created_at))
        .load::<models::UserDTO>(conn)
        .optional()?)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_user(
    mut nu: Rc<models::NewUser>,
    conn: &SqliteConnection,
) -> Result<models::UserDTO, errors::DomainError> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::users::dsl::*;

    // let new_user = models::User {
    //     id: Uuid::new_v4().to_string(),
    //     name: nu.name.to_string(),
    // };

    // let x = users.load::<models::User>(conn).optional();
    // let target = users.find("4");
    // let test_user = models::User {
    //     id: "5".to_owned(),
    //     name: "who".to_owned(),
    // };
    // let update_result = diesel::update(target).set(&test_user).execute(conn);

    // let mut nu2 = nu.clone();
    let mut nu2 = Rc::make_mut(&mut nu);
    nu2.password = hash(nu2.password.clone(), DEFAULT_COST)?;

    diesel::insert_into(users)
        .values(nu.as_ref())
        .execute(conn)?;
    let user =
        _get_user_by_name(nu.name.clone()).first::<models::UserDTO>(conn)?;
    Ok(user)
}

pub fn verify_password(
    user_name: String,
    given_password: String,
    conn: &SqliteConnection,
) -> Result<bool, errors::DomainError> {
    use crate::schema::users::dsl::*;
    let password_hash = users
        .select(password)
        .filter(name.eq(user_name))
        .first::<String>(conn)?;
    Ok(verify(given_password.as_str(), password_hash.as_str())?)
}
fn _get_user_by_name(
    user_name: String,
) -> crate::schema::users::BoxedQuery<
    'static,
    Sqlite,
    (diesel::sql_types::Text, diesel::sql_types::Timestamp),
> {
    use crate::schema::users::dsl::*;
    users
        .select((name, created_at))
        .filter(name.eq(user_name))
        .into_boxed()
}
