use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel_tracing::pg::InstrumentedPgConnection;
use r2d2::PooledConnection;

use crate::errors::DomainError;
use crate::models::misc::Pagination;
use crate::models::roles::{NewUserRole, RoleEnum, RoleId};
use crate::models::users::{
    NewUser, Password, User, UserAuthDetails, UserAuthDetailsWithRoles, UserId,
    UserWithRoles, Username,
};
use bcrypt::hash;
use do_notation::m;
use validators::prelude::*;

pub fn get_roles_for_user(
    uid: &UserId,
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<Vec<RoleEnum>, DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users_roles::dsl as users_roles;
    Ok(users_roles::users_roles
        .inner_join(roles::roles)
        .select(roles::role_name)
        .filter(users_roles::user_id.eq(uid))
        .load::<RoleEnum>(conn)?)
}

pub fn get_roles_for_users(
    users: Vec<User>,
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<Vec<UserWithRoles>, DomainError> {
    users
        .into_iter()
        .map(|user| {
            get_roles_for_user(&user.id, conn)
                .map(|roles| UserWithRoles::from_user(&user, &roles))
                .map_err(DomainError::from)
        })
        .collect::<Result<Vec<UserWithRoles>, DomainError>>()
}

pub fn find_user_by_uid(
    uid: &UserId,
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<Option<UserWithRoles>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let mb_user = users::users
            .select((users::id, users::username, users::created_at))
            .filter(users::id.eq(uid))
            .first::<User>(conn)
            .optional()?;

        let roles = get_roles_for_user(uid, conn)?;

        Ok(mb_user.map(|user| UserWithRoles::from_user(&user, &roles)))
    })
}

pub fn find_user_by_name(
    user_name: &Username,
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<Option<UserWithRoles>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let mb_user = users::users
            .select((users::id, users::username, users::created_at))
            .filter(users::username.eq(user_name))
            .first::<User>(conn)
            .optional()?;

        let roles = match &mb_user {
            Some(user) => Some(get_roles_for_user(&user.id, conn)?),
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
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<Option<UserAuthDetailsWithRoles>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let mb_user = users::users
            .select((users::id, users::username, users::password))
            .filter(users::username.eq(user_name))
            .first::<UserAuthDetails>(conn)
            .optional()?;

        let roles = match &mb_user {
            Some(user) => Some(get_roles_for_user(&user.id, conn)?),
            None => None,
        };

        let mb_user_with_roles = m! {
            user <- mb_user;
            roles <- roles;
            Some(UserAuthDetailsWithRoles::from_user(user, roles))
        };

        Ok(mb_user_with_roles)
    })
}

pub fn get_all_users(
    pagination: &Pagination,
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<Vec<UserWithRoles>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let users = users::users
            .select((users::id, users::username, users::created_at))
            .order_by(users::created_at)
            .offset(pagination.calc_offset().as_uint().into())
            .limit(pagination.limit.as_uint().into())
            .load::<User>(conn)?;

        get_roles_for_users(users, conn)
    })
}

pub fn search_users(
    query: &str,
    pagination: &Pagination,
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<Vec<UserWithRoles>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let users = users::users
            .select((users::id, users::username, users::created_at))
            .filter(users::username.like(format!("%{}%", query)))
            .order_by(users::created_at)
            .offset(pagination.calc_offset().as_uint().into())
            .limit(pagination.limit.as_uint().into())
            .load::<User>(conn)?;

        get_roles_for_users(users, conn)
    })
}

pub fn insert_new_user(
    nu: NewUser,
    role: RoleEnum,
    hash_cost: u32,
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<UserWithRoles, DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    let nu = {
        let mut nu2 = nu;
        let hash = hash(nu2.password.as_str(), hash_cost)?;
        nu2.password = Password::parse_string(hash).map_err(|err| {
            DomainError::new_field_validation_error(err.to_string())
        })?;
        nu2
    };
    conn.transaction(|conn| {
        let _ = diesel::insert_into(users::users)
            .values(&nu)
            .execute(conn)?;
        let role_id = roles::roles
            .select(roles::id)
            .filter(roles::role_name.eq(role))
            .first::<RoleId>(conn)?;
        let user = users::users
            .select((users::id, users::username, users::created_at))
            .filter(users::username.eq(nu.username))
            .first::<User>(conn)?;

        let _ = diesel::insert_into(users_roles::users_roles)
            .values(NewUserRole {
                user_id: user.id,
                role_id,
            })
            .execute(conn)?;

        let roles = get_roles_for_user(&user.id, conn)?;

        Ok(UserWithRoles {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            roles,
        })
    })
}

pub fn insert_new_regular_user(
    nu: NewUser,
    hash_cost: u32,
    conn: &mut PooledConnection<ConnectionManager<InstrumentedPgConnection>>,
) -> Result<UserWithRoles, DomainError> {
    insert_new_user(nu, RoleEnum::RoleUser, hash_cost, conn)
}
