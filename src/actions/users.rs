use diesel::prelude::*;

use crate::models::{self, Pagination, UserId, Username};
use crate::{errors, models::Password};
use bcrypt::{hash, DEFAULT_COST};
use validators::prelude::*;

pub fn find_user_by_uid(
    uid: &UserId,
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

pub fn find_user_by_name(
    user_name: &Username,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Option<models::User>, errors::DomainError> {
    use crate::schema::users::dsl::*;
    let maybe_user = query::_get_user_by_name()
        .filter(name.eq(user_name))
        .first::<models::User>(conn)
        .optional();

    Ok(maybe_user?)
}

// def findAll(userId: Long, limit: Int, offset: Int) = db.run {
//     for {
//       comments <- query.filter(_.creatorId === userId)
//                        .sortBy(_.createdAt)
//                        .drop(offset).take(limit)
//                        .result
//       numberOfComments <- query.filter(_.creatorId === userId).length.result
//     } yield PaginatedResult(
//         totalCount = numberOfComments,
//         entities = comments.toList,
//         hasNextPage = numberOfComments - (offset + limit) > 0
//     )
//   }

pub fn get_all_users(
    pagination: &Pagination,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Vec<models::User>, errors::DomainError> {
    Ok(query::_paginate_result(pagination).load::<models::User>(conn)?)
}

pub fn search_users(
    query: &str,
    pagination: &Pagination,
    conn: &impl diesel::Connection<Backend = diesel::sqlite::Sqlite>,
) -> Result<Vec<models::User>, errors::DomainError> {
    use crate::schema::users::dsl::*;
    Ok(query::_paginate_result(pagination)
        .filter(name.like(format!("%{}%", query)))
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
            hash(nu2.password.as_str(), hash_cost.unwrap_or(DEFAULT_COST))?;
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

mod query {
    use super::*;
    use diesel::sql_types::Integer;
    use diesel::sql_types::Text;
    use diesel::sql_types::Timestamp;
    use diesel::sqlite::Sqlite;

    /// <'a, B, T> where a = lifetime, B = Backend, T = SQL data types
    type Query<'a, B, T> = crate::schema::users::BoxedQuery<'a, B, T>;

    pub fn _get_user_by_name(
    ) -> Query<'static, Sqlite, (Integer, Text, Text, Timestamp)> {
        use crate::schema::users::dsl::*;
        users.into_boxed()
    }

    pub fn _paginate_result(
        pagination: &Pagination,
    ) -> Query<'static, Sqlite, (Integer, Text, Text, Timestamp)> {
        use crate::schema::users::dsl::*;
        users
            .order_by(created_at)
            .offset(pagination.calc_offset().as_uint().into())
            .limit(pagination.limit.as_uint().into())
            .into_boxed()
    }
}
