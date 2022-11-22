use diesel::prelude::*;

use crate::models::{self, Pagination, User, UserId, Username};
use crate::{errors, models::Password};
use bcrypt::{hash, DEFAULT_COST};
use validators::prelude::*;

pub fn find_user_by_uid(
    uid: &UserId,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Option<models::User>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;

    let maybe_user = users::users
        .inner_join(roles::roles)
        .select((
            users::id,
            users::username,
            users::created_at,
            roles::role_name,
        ))
        .filter(users::id.eq(uid))
        .first::<User>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn find_user_by_name2(
    user_name: &Username,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Option<models::User>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    let maybe_user = users::users
        .inner_join(roles::roles)
        .select((
            users::id,
            users::username,
            users::created_at,
            roles::role_name,
        ))
        .filter(users::username.eq(user_name))
        .first::<models::User>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn get_user_auth_details(
    user_name: &Username,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Option<models::UserAuthDetails>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    let maybe_user = users::users
        .inner_join(roles::roles)
        .select((
            users::id,
            users::username,
            users::password,
            roles::role_name,
        ))
        .filter(users::username.eq(user_name))
        .first::<models::UserAuthDetails>(conn)
        .optional();

    Ok(maybe_user?)
}

pub fn get_all_users(
    pagination: &Pagination,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Vec<models::User>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    Ok(users::users
        .inner_join(roles::roles)
        .select((
            users::id,
            users::username,
            users::created_at,
            roles::role_name,
        ))
        .order_by(users::created_at)
        .offset(pagination.calc_offset().as_uint().into())
        .limit(pagination.limit.as_uint().into())
        .load::<models::User>(conn)?)
}

pub fn search_users(
    query: &str,
    pagination: &Pagination,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Vec<models::User>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    Ok(users::users
        .inner_join(roles::roles)
        .select((
            users::id,
            users::username,
            users::created_at,
            roles::role_name,
        ))
        .order_by(users::created_at)
        .offset(pagination.calc_offset().as_uint().into())
        .limit(pagination.limit.as_uint().into())
        .filter(users::username.like(format!("%{}%", query)))
        .load::<models::User>(conn)?)
}

pub fn insert_new_user(
    nu: models::NewUser,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
    hash_cost: Option<u32>,
) -> Result<models::User, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    let nu = {
        let mut nu2 = nu;
        let hash =
            hash(nu2.password.as_str(), hash_cost.unwrap_or(DEFAULT_COST))?;
        nu2.password = Password::parse_string(hash).map_err(|err| {
            errors::DomainError::new_field_validation_error(err.to_string())
        })?;
        nu2
    };

    diesel::insert_into(users::users)
        .values(&nu)
        .execute(conn)?;
    let user = users::users
        .inner_join(roles::roles)
        .select((
            users::id,
            users::username,
            users::created_at,
            roles::role_name,
        ))
        .filter(users::username.eq(nu.username.as_str()))
        .first::<models::User>(conn)?;

    Ok(user)
}
