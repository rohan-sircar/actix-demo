#![allow(unused_imports)]
table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;
    use crate::models::misc::*;

    jobs (id) {
        id -> Int4,
        job_id -> Uuid,
        started_by -> Int4,
        status -> Job_status,
        status_message -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;
    use crate::models::misc::*;

    roles (id) {
        id -> Int4,
        role_name -> Role_name,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::roles::*;
    use crate::models::misc::*;

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
    use crate::models::misc::*;

    users_roles (id) {
        id -> Int4,
        user_id -> Int4,
        role_id -> Int4,
    }
}

joinable!(jobs -> users (started_by));
joinable!(users_roles -> roles (role_id));
joinable!(users_roles -> users (user_id));

allow_tables_to_appear_in_same_query!(jobs, roles, users, users_roles,);
