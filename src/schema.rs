#![allow(unused_imports)]
table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;

    roles (id) {
        id -> Int4,
        role_name -> Role_name,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;

    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;

    users_roles (id) {
        id -> Int4,
        user_id -> Int4,
        role_id -> Int4,
    }
}

joinable!(users_roles -> roles (role_id));
joinable!(users_roles -> users (user_id));

allow_tables_to_appear_in_same_query!(roles, users, users_roles,);
