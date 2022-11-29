use std::str::FromStr;

use diesel::prelude::*;

use crate::models::roles::{NewUserRole, RoleEnum, RoleId};
use crate::models::{self, Pagination, User, UserId, UserWithRoles, Username};
use crate::{errors, models::Password};
use bcrypt::hash;
use do_notation::m;
use validators::prelude::*;

pub fn find_user_by_uid(
    uid: &UserId,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Option<models::UserWithRoles>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    conn.transaction(|| {
        let mb_user = users::users
            .select((users::id, users::username, users::created_at))
            .filter(users::id.eq(uid))
            .first::<User>(conn)
            .optional()?;

        let roles = users_roles::users_roles
            .inner_join(roles::roles)
            .select(roles::role_name)
            .filter(users_roles::user_id.eq(uid))
            .load::<RoleEnum>(conn)?;

        Ok(mb_user.map(|user| UserWithRoles {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            roles,
        }))
    })
}

pub fn find_user_by_name(
    user_name: &Username,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Option<models::UserWithRoles>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    conn.transaction(|| {
        let mb_user = users::users
            .select((users::id, users::username, users::created_at))
            .filter(users::username.eq(user_name))
            .first::<models::User>(conn)
            .optional()?;

        let roles = match &mb_user {
            Some(user) => Some(
                users_roles::users_roles
                    .inner_join(roles::roles)
                    .select(roles::role_name)
                    .filter(users_roles::user_id.eq(&user.id))
                    .load::<RoleEnum>(conn)?,
            ),
            None => None,
        };

        let mb_user_with_roles = m! {
            user <- mb_user;
            roles <- roles;
            Some(UserWithRoles {
                id: user.id,
                username: user.username,
                created_at: user.created_at,
                roles,
            })
        };

        Ok(mb_user_with_roles)
    })
}

pub fn get_user_auth_details(
    user_name: &Username,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Option<models::UserAuthDetailsWithRoles>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    conn.transaction(|| {
        let mb_user = users::users
            .select((users::id, users::username, users::password))
            .filter(users::username.eq(user_name))
            .first::<models::UserAuthDetails>(conn)
            .optional()?;

        let roles = match &mb_user {
            Some(user) => Some(
                users_roles::users_roles
                    .inner_join(roles::roles)
                    .select(roles::role_name)
                    .filter(users_roles::user_id.eq(&user.id))
                    .load::<RoleEnum>(conn)?,
            ),
            None => None,
        };

        let mb_user_with_roles = m! {
            user <- mb_user;
            roles <- roles;
            Some(models::UserAuthDetailsWithRoles {
                id: user.id,
                username: user.username,
                password: user.password,
                roles,
            })
        };

        Ok(mb_user_with_roles)
    })
}

pub fn get_all_users(
    pagination: &Pagination,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Vec<models::UserWithRoles>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    conn.transaction(|| {
        let users = users::users
            .select((users::id, users::username, users::created_at))
            .order_by(users::created_at)
            .offset(pagination.calc_offset().as_uint().into())
            .limit(pagination.limit.as_uint().into())
            .load::<models::User>(conn)?;

        users
            .into_iter()
            .map(|user| {
                users_roles::users_roles
                    .inner_join(roles::roles)
                    .select(roles::role_name)
                    .filter(users_roles::user_id.eq(&user.id))
                    .load::<RoleEnum>(conn)
                    .map(|roles| UserWithRoles {
                        id: user.id,
                        username: user.username,
                        created_at: user.created_at,
                        roles,
                    })
                    .map_err(errors::DomainError::from)
            })
            .collect::<Result<Vec<UserWithRoles>, errors::DomainError>>()
    })
}

pub fn search_users(
    query: &str,
    pagination: &Pagination,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Vec<models::UserWithRoles>, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    conn.transaction(|| {
        let users = users::users
            .select((users::id, users::username, users::created_at))
            .filter(users::username.like(format!("%{}%", query)))
            .order_by(users::created_at)
            .offset(pagination.calc_offset().as_uint().into())
            .limit(pagination.limit.as_uint().into())
            .load::<models::User>(conn)?;

        users
            .into_iter()
            .map(|user| {
                users_roles::users_roles
                    .inner_join(roles::roles)
                    .select(roles::role_name)
                    .filter(users_roles::user_id.eq(&user.id))
                    .load::<RoleEnum>(conn)
                    .map(|roles| UserWithRoles {
                        id: user.id,
                        username: user.username,
                        created_at: user.created_at,
                        roles,
                    })
                    .map_err(errors::DomainError::from)
            })
            .collect::<Result<Vec<UserWithRoles>, errors::DomainError>>()
    })
}

pub fn insert_new_user(
    nu: models::NewUser,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
    hash_cost: u32,
) -> Result<models::UserWithRoles, errors::DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    let nu = {
        let mut nu2 = nu;
        let hash = hash(nu2.password.as_str(), hash_cost)?;
        nu2.password = Password::parse_string(hash).map_err(|err| {
            errors::DomainError::new_field_validation_error(err.to_string())
        })?;
        nu2
    };
    conn.transaction(|| {
        let _ = diesel::insert_into(users::users)
            .values(&nu)
            .execute(conn)?;

        let user = users::users
            .select((users::id, users::username, users::created_at))
            .filter(users::username.eq(nu.username))
            .first::<models::User>(conn)?;

        let _ = diesel::insert_into(users_roles::users_roles)
            .values(NewUserRole {
                user_id: user.id.clone(),
                role_id: RoleId::from_str("3").unwrap(), //TODO fix this
            })
            .execute(conn)?;

        let roles = users_roles::users_roles
            .inner_join(roles::roles)
            .select(roles::role_name)
            .filter(users_roles::user_id.eq(&user.id))
            .load::<RoleEnum>(conn)?;

        Ok(UserWithRoles {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            roles,
        })
    })
}
