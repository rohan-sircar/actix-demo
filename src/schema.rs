// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "role_name"))]
    pub struct RoleName;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RoleName;

    roles (id) {
        id -> Int4,
        name -> RoleName,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        role_id -> Int4,
    }
}

diesel::joinable!(users -> roles (role_id));

diesel::allow_tables_to_appear_in_same_query!(
    roles,
    users,
);
