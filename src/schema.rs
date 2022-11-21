// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;

    roles (id) {
        id -> Integer,
        role_name -> RoleEnum,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;

    users2 (id) {
        id -> Integer,
        name -> Text,
        password -> Text,
        role_id -> Integer,
        created_at -> Timestamp,
    }
}

diesel::joinable!(users2 -> roles (role_id));

diesel::allow_tables_to_appear_in_same_query!(roles, users2,);
