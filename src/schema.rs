// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_status"))]
    pub struct JobStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "like_direction"))]
    pub struct LikeDirection;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "oauth_provider_type"))]
    pub struct OauthProviderType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pet_gender"))]
    pub struct PetGender;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "role_name"))]
    pub struct RoleName;
}

diesel::table! {
    email_verification_tokens (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 64]
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        used -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobStatus;

    jobs (id) {
        id -> Int4,
        job_id -> Uuid,
        started_by -> Nullable<Int4>,
        status -> JobStatus,
        status_message -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LikeDirection;

    likes (id) {
        id -> Int4,
        user_id -> Int4,
        pet_owner_id -> Int4,
        pet_id -> Int4,
        direction -> LikeDirection,
        is_match -> Bool,
        created_at -> Timestamptz,
        matched_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    password_reset_tokens (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 64]
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        used -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    personality_traits (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    pet_images (id) {
        id -> Int4,
        uuid -> Uuid,
        pet_id -> Int4,
        thumbnail_key -> Text,
        medium_key -> Text,
        original_key -> Text,
        #[max_length = 10]
        format -> Varchar,
        is_primary -> Bool,
        sort_order -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    pet_personality_traits (pet_id, trait_id) {
        pet_id -> Int4,
        trait_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PetGender;

    pets (id) {
        id -> Int4,
        pet_uuid -> Uuid,
        user_id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 50]
        species -> Varchar,
        #[max_length = 200]
        breed -> Nullable<Varchar>,
        date_of_birth -> Nullable<Date>,
        gender -> Nullable<PetGender>,
        weight -> Nullable<Float8>,
        #[max_length = 200]
        color_markings -> Nullable<Varchar>,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    profiles (id) {
        id -> Int4,
        user_id -> Int4,
        bio -> Nullable<Text>,
        #[max_length = 100]
        display_name -> Nullable<Varchar>,
        #[max_length = 200]
        location -> Nullable<Varchar>,
        #[max_length = 500]
        website_url -> Nullable<Varchar>,
        #[max_length = 100]
        social_github -> Nullable<Varchar>,
        #[max_length = 100]
        social_twitter -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        avatar_url -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RoleName;

    roles (id) {
        id -> Int4,
        role_name -> RoleName,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OauthProviderType;

    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
        #[max_length = 255]
        email -> Varchar,
        oauth_provider -> Nullable<OauthProviderType>,
        #[max_length = 128]
        oauth_uid -> Nullable<Varchar>,
        user_uuid -> Uuid,
        email_verified -> Bool,
    }
}

diesel::table! {
    users_roles (id) {
        id -> Int4,
        user_id -> Int4,
        role_id -> Int4,
    }
}

diesel::joinable!(email_verification_tokens -> users (user_id));
diesel::joinable!(jobs -> users (started_by));
diesel::joinable!(likes -> pets (pet_id));
diesel::joinable!(password_reset_tokens -> users (user_id));
diesel::joinable!(pet_images -> pets (pet_id));
diesel::joinable!(pet_personality_traits -> personality_traits (trait_id));
diesel::joinable!(pet_personality_traits -> pets (pet_id));
diesel::joinable!(pets -> users (user_id));
diesel::joinable!(profiles -> users (user_id));
diesel::joinable!(users_roles -> roles (role_id));
diesel::joinable!(users_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    email_verification_tokens,
    jobs,
    likes,
    password_reset_tokens,
    personality_traits,
    pet_images,
    pet_personality_traits,
    pets,
    profiles,
    roles,
    users,
    users_roles,
);
