use diesel::prelude::*;

use crate::models;
use crate::{errors, models::Password};
use bcrypt::{hash, verify, DEFAULT_COST};
use validators::prelude::*;

pub fn find_user_by_uid(
    uid: i32,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Option<models::User>, errors::DomainError> {
    use crate::schema::users::dsl::*;

    let maybe_user = users
        .select(users::all_columns())
        .find(uid)
        .first::<models::User>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn _find_user_by_name(
    user_name: String,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Option<models::User>, errors::DomainError> {
    use crate::schema::users::dsl::*;
    let maybe_user = query::_get_user_by_name()
        .filter(name.eq(user_name))
        .first::<models::User>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn get_all(
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Vec<models::User>, errors::DomainError> {
    use crate::schema::users::dsl::*;
    Ok(users
        .select(users::all_columns())
        .load::<models::User>(conn)?)
}

pub fn insert_new_user(
    nu: models::NewUser,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
    hash_cost: Option<u32>,
) -> Result<models::User, errors::DomainError> {
    use crate::schema::users::dsl::*;
    let nu = {
        let mut nu2 = nu;
        let hash =
            hash(&nu2.password.as_str(), hash_cost.unwrap_or(DEFAULT_COST))?;
        nu2.password = Password::parse_string(hash).map_err(|err| {
            errors::DomainError::new_field_validation_error(err.to_string())
        })?;
        nu2
    };

    diesel::insert_into(users).values(&nu).execute(conn)?;
    let user = query::_get_user_by_name()
        .filter(name.eq(nu.name.as_str()))
        .first::<models::User>(conn)?;

    Ok(user)
}

//TODO: Add newtype here
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
    use diesel::sql_types::Integer;
    use diesel::sql_types::Text;
    use diesel::sql_types::Timestamp;
    use diesel::sqlite::Sqlite;

    /// <'a, B, T> where a = lifetime, B = Backend, T = SQL data types
    type Query<'a, B, T> = crate::schema::users::BoxedQuery<'a, B, T>;

    pub fn _get_user_by_name(
    ) -> Query<'static, Sqlite, (Integer, Text, Text, Timestamp)> {
        use crate::schema::users::dsl::*;
        users
            .select(users::all_columns())
            // .filter(name.eq(user_name))
            .into_boxed()
    }
}
