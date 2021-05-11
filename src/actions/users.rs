use diesel::prelude::*;

use crate::errors;
use crate::models;
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn find_user_by_uid(
    uid: i32,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Option<models::UserDto>, errors::DomainError> {
    use crate::schema::users::dsl::*;

    let maybe_user = users
        .select((name, created_at))
        .find(uid)
        .first::<models::UserDto>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn _find_user_by_name(
    user_name: String,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Option<models::UserDto>, errors::DomainError> {
    let maybe_user = query::_get_user_by_name(&user_name)
        .first::<models::UserDto>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn get_all(
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Vec<models::UserDto>, errors::DomainError> {
    use crate::schema::users::dsl::*;
    Ok(users
        .select((name, created_at))
        .load::<models::UserDto>(conn)?)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_user(
    nu: models::NewUser,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<models::UserDto, errors::DomainError> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::users::dsl::*;
    let nu = {
        let mut nu2 = nu;
        nu2.password = hash(&nu2.password, DEFAULT_COST)?;
        nu2
    };

    diesel::insert_into(users).values(&nu).execute(conn)?;
    let user =
        query::_get_user_by_name(&nu.name).first::<models::UserDto>(conn)?;
    Ok(user)
}

pub fn verify_password(
    user_name: &str,
    given_password: &str,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
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

    pub fn _get_user_by_name(
        user_name: &str,
    ) -> Query<Sqlite, (Text, Timestamp)> {
        use crate::schema::users::dsl::*;
        users
            .select((name, created_at))
            .filter(name.eq(user_name))
            .into_boxed()
    }
}
