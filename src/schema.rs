table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;

    roles (id) {
        id -> Int4,
        name -> Role_name,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;

    users (id) {
        id -> Int4,
        name -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        role_id -> Int4,
    }
}

joinable!(users -> roles (role_id));

allow_tables_to_appear_in_same_query!(
    roles,
    users,
);
