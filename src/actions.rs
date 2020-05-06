use diesel::prelude::*;

use crate::models;

pub fn find_user_by_uid(
    uid: i32,
    conn: &SqliteConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let maybe_user = users.find(uid).first::<models::User>(conn).optional();

    // Ok(user)
    maybe_user
}

pub fn find_user_by_name(
    user_name: String,
    conn: &SqliteConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let maybe_user = users
        .filter(name.eq(user_name))
        .first::<models::User>(conn)
        .optional();

    maybe_user
}

pub fn get_all(
    conn: &SqliteConnection,
) -> Result<Option<Vec<models::User>>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users.load::<models::User>(conn).optional()
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_user(
    nu: &models::NewUser,
    conn: &SqliteConnection,
) -> Result<models::User, diesel::result::Error> {
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

    diesel::insert_into(users).values(nu).execute(conn)?;
    let user = users
        .filter(name.eq(nu.name.clone()))
        .first::<models::User>(conn);
    user
    // Ok(nu.clone())
}
