use diesel::prelude::*;

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
    let maybe_user = query::_get_user_by_name(&user_name)
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
    let mut nu2 = Rc::make_mut(&mut nu);
    nu2.password = hash(&nu2.password, DEFAULT_COST)?;

    diesel::insert_into(users)
        .values(nu.as_ref())
        .execute(conn)?;
    let user =
        query::_get_user_by_name(&nu.name).first::<models::UserDTO>(conn)?;
    Ok(user)
}

pub fn verify_password<'a>(
    user_name: &'a String,
    given_password: &'a String,
    conn: &SqliteConnection,
) -> Result<bool, errors::DomainError> {
    use crate::schema::users::dsl::*;
    let password_hash = users
        .select(password)
        .filter(name.eq(user_name))
        .first::<String>(conn)?;
    Ok(verify(given_password, password_hash.as_str())?)
}

mod query {
    use diesel::prelude::*;
    use diesel::sql_types::Text;
    use diesel::sql_types::Timestamp;
    use diesel::sqlite::Sqlite;

    /// <'a, B, T> where a = lifetime, B = Backend, T = SQL data types
    type Query<'a, B, T> = crate::schema::users::BoxedQuery<'a, B, T>;

    pub fn _get_user_by_name<'a>(
        user_name: &'a String,
    ) -> Query<'a, Sqlite, (Text, Timestamp)> {
        use crate::schema::users::dsl::*;
        users
            .select((name, created_at))
            .filter(name.eq(user_name))
            .into_boxed()
    }
}
