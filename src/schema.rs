// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "adoption_status_type"))]
    pub struct AdoptionStatusType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "barking_level_type"))]
    pub struct BarkingLevelType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "coat_type"))]
    pub struct CoatType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "energy_level_type"))]
    pub struct EnergyLevelType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "gender_type"))]
    pub struct GenderType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_status"))]
    pub struct JobStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pet_type"))]
    pub struct PetType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "role_name"))]
    pub struct RoleName;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "size_type"))]
    pub struct SizeType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "trainability_type"))]
    pub struct TrainabilityType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobStatus;

    jobs (id) {
        id -> Int4,
        job_id -> Uuid,
        started_by -> Int4,
        status -> JobStatus,
        status_message -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::EnergyLevelType;
    use super::sql_types::TrainabilityType;
    use super::sql_types::BarkingLevelType;

    pet_activities (id) {
        id -> Int4,
        pet_profile_uuid -> Uuid,
        favorite_activities -> Nullable<Array<Nullable<Text>>>,
        likes -> Nullable<Array<Nullable<Text>>>,
        dislikes -> Nullable<Array<Nullable<Text>>>,
        energy_level -> Nullable<EnergyLevelType>,
        trainability -> Nullable<TrainabilityType>,
        barking_level -> Nullable<BarkingLevelType>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AdoptionStatusType;

    pet_adoption_details (id) {
        id -> Int4,
        pet_profile_uuid -> Uuid,
        special_needs -> Nullable<Bool>,
        special_needs_description -> Nullable<Text>,
        adoption_status -> Nullable<AdoptionStatusType>,
        #[max_length = 100]
        shelter_name -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PetType;
    use super::sql_types::GenderType;
    use super::sql_types::SizeType;
    use super::sql_types::CoatType;

    pet_basic_info (id) {
        id -> Int4,
        uuid -> Uuid,
        user_id -> Int4,
        #[max_length = 100]
        pet_name -> Varchar,
        pet_type -> PetType,
        #[max_length = 100]
        breed -> Varchar,
        age -> Int4,
        weight_kg -> Float4,
        gender -> GenderType,
        size -> Nullable<SizeType>,
        #[max_length = 50]
        color -> Nullable<Varchar>,
        coat_type -> Nullable<CoatType>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    pet_location_owner (id) {
        id -> Int4,
        pet_profile_uuid -> Uuid,
        #[max_length = 100]
        owner_name -> Varchar,
        #[max_length = 100]
        location -> Varchar,
        address -> Nullable<Text>,
        lat -> Nullable<Numeric>,
        lng -> Nullable<Numeric>,
    }
}

diesel::table! {
    pet_personality_traits (id) {
        id -> Int4,
        pet_profile_uuid -> Uuid,
        bio -> Nullable<Text>,
        personality_traits -> Nullable<Array<Nullable<Text>>>,
        good_with_dogs -> Nullable<Bool>,
        good_with_cats -> Nullable<Bool>,
        good_with_kids -> Nullable<Bool>,
        house_trained -> Nullable<Bool>,
        vaccinated -> Nullable<Bool>,
        spayed_neutered -> Nullable<Bool>,
        microchipped -> Nullable<Bool>,
    }
}

diesel::table! {
    pet_profile_images (id) {
        id -> Int4,
        pet_profile_uuid -> Uuid,
        image_url -> Text,
        is_primary -> Nullable<Bool>,
        sort_order -> Nullable<Int4>,
        created_at -> Timestamp,
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
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users_roles (id) {
        id -> Int4,
        user_id -> Int4,
        role_id -> Int4,
    }
}

diesel::joinable!(jobs -> users (started_by));
diesel::joinable!(pet_basic_info -> users (user_id));
diesel::joinable!(users_roles -> roles (role_id));
diesel::joinable!(users_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    jobs,
    pet_activities,
    pet_adoption_details,
    pet_basic_info,
    pet_location_owner,
    pet_personality_traits,
    pet_profile_images,
    roles,
    users,
    users_roles,
);
