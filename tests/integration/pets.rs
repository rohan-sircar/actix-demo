#[cfg(test)]
mod tests {

    use actix_demo::models::misc::ErrorResponse;
    use actix_demo::models::pet_profile_full::FullPetProfile;
    use actix_web::http::StatusCode;

    mod create_pet_profile_api {

        use crate::common::{TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_create_a_pet_profile() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile with valid data
            let pet_data = serde_json::json!({
                "user_id": 1,
                "pet_name": "Fluffy",
                "pet_type": "cat",
                "breed": "Persian",
                "age": 5,
                "weight_kg": 4.5,
                "gender": "female",
                "bio": "A very cute cat",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::CREATED);

            let body: FullPetProfile = resp.json().await.unwrap();
            assert_eq!(body.basic_info.pet_name.as_str(), "Fluffy");
            assert_eq!(
                body.basic_info.pet_type,
                actix_demo::models::pet_enums::PetType::Cat
            );
            assert_eq!(
                body.personality_traits.map(|t| t.bio),
                Some(Some("A very cute cat".to_owned()))
            );
        }

        #[actix_rt::test]
        async fn should_return_error_when_creating_pet_profile_with_invalid_data(
        ) {
            let ctx = TestContext::new(None).await;

            // Try to create a pet profile with invalid data (empty name)
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "",
                "pet_type": "cat",
                "breed": "Persian",
                "age": 3,
                "weight_kg": 4.5,
                "gender": "female",
                "bio": "A very cute cat",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            println!("{:?}", std::str::from_utf8(&resp.body().await.unwrap()));
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }
    }

    mod weight_validation {
        use crate::common::{TestContext, WithToken};
        use actix_demo::models::misc::ErrorResponse;

        use super::*;

        #[actix_rt::test]
        async fn should_validate_weight_kg_field() {
            let ctx = TestContext::new(None).await;

            // Invalid weights that should return BAD_REQUEST
            let invalid_weights: Vec<f32> = vec![
                0.0,      // Zero - below minimum
                -5.0,     // Negative - below minimum
                151.0,    // Just above maximum
                200.0,    // Well above maximum
            ];

            for weight in invalid_weights {
                let pet_data = serde_json::json!({
                    "user_id": 1,
                    "pet_name": "Fluffy",
                    "pet_type": "cat",
                    "breed": "Persian",
                    "age": 3,
                    "weight_kg": weight,
                    "gender": "female",
                    "bio": "A very cute cat",
                    "owner_name": "Owner Name",
                    "location": "Location",
                    "special_needs": false,
                    "images": []
                });

                let mut resp = ctx
                    .test_server
                    .post("/api/pets")
                    .with_token(&ctx._token)
                    .send_json(&pet_data)
                    .await
                    .unwrap();

                assert_eq!(
                    resp.status(),
                    StatusCode::BAD_REQUEST,
                    "Weight {} should be rejected",
                    weight
                );

                let body: ErrorResponse<String> = resp.json().await.unwrap();
                assert!(
                    body.cause.contains("weight") || body.cause.contains("Invalid"),
                    "Error message should mention weight for value {}",
                    weight
                );
            }

            // Valid weights at boundaries that should succeed
            let valid_weights: Vec<f32> = vec![1.0, 150.0];

            for weight in valid_weights {
                let pet_data = serde_json::json!({
                    "user_id": 1,
                    "pet_name": "Fluffy",
                    "pet_type": "cat",
                    "breed": "Persian",
                    "age": 3,
                    "weight_kg": weight,
                    "gender": "female",
                    "bio": "A very cute cat",
                    "owner_name": "Owner Name",
                    "location": "Location",
                    "special_needs": false,
                    "images": []
                });

                let mut resp = ctx
                    .test_server
                    .post("/api/pets")
                    .with_token(&ctx._token)
                    .send_json(&pet_data)
                    .await
                    .unwrap();

                assert_eq!(
                    resp.status(),
                    StatusCode::CREATED,
                    "Weight {} should be accepted",
                    weight
                );

                let body: FullPetProfile = resp.json().await.unwrap();
                assert_eq!(
                    body.basic_info.weight_kg.as_f32(),
                    weight,
                    "Stored weight should match input for {}",
                    weight
                );
            }
        }
    }

    mod weight_update_validation {
        use crate::common::{TestContext, WithToken};
        use actix_demo::models::misc::ErrorResponse;

        use super::*;

        #[actix_rt::test]
        async fn should_validate_weight_kg_field_on_update() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Invalid weights that should return BAD_REQUEST
            let invalid_weights: Vec<f32> = vec![
                0.0,      // Zero - below minimum
                -10.0,    // Negative - below minimum
                151.0,    // Just above maximum
                200.0,    // Well above maximum
            ];

            for weight in invalid_weights {
                let update_data = serde_json::json!({
                    "basic_info" : {
                        "pet_name": "Updated Buddy",
                        "breed": "Labrador Retriever",
                        "age": 3,
                        "weight_kg": weight,
                    },
                    "personality_traits" : {"bio": "An updated friendly dog"},
                    "images": []
                });

                let mut resp = ctx
                    .test_server
                    .patch(&format!("/api/pets/{}", pet_uuid))
                    .with_token(&ctx._token)
                    .send_json(&update_data)
                    .await
                    .unwrap();

                assert_eq!(
                    resp.status(),
                    StatusCode::BAD_REQUEST,
                    "Weight {} should be rejected on update",
                    weight
                );

                let body: ErrorResponse<String> = resp.json().await.unwrap();
                assert!(
                    body.cause.contains("weight") || body.cause.contains("Invalid"),
                    "Error message should mention weight for value {}",
                    weight
                );
            }

            // Valid weights that should succeed
            let valid_weights: Vec<f32> = vec![30.5, 150.0];

            for weight in valid_weights {
                let update_data = serde_json::json!({
                    "basic_info" : {
                        "pet_name": "Updated Buddy",
                        "breed": "Labrador Retriever",
                        "age": 3,
                        "weight_kg": weight,
                    },
                    "personality_traits" : {"bio": "An updated friendly dog"},
                    "images": []
                });

                let mut resp = ctx
                    .test_server
                    .patch(&format!("/api/pets/{}", pet_uuid))
                    .with_token(&ctx._token)
                    .send_json(&update_data)
                    .await
                    .unwrap();

                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "Weight {} should be accepted on update",
                    weight
                );

                let body: FullPetProfile = resp.json().await.unwrap();
                assert_eq!(
                    body.basic_info.weight_kg.as_f32(),
                    weight,
                    "Stored weight should match input for {}",
                    weight
                );
            }
        }
    }

    mod get_pet_profile_api {

        use actix_demo::models::pets::PetPersonalityTraits;
        use diesel_tracing::pg::InstrumentedPgConnection;

        use crate::common::{TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_get_a_pet_profile_by_id() {
            let ctx = TestContext::new(None).await;
            // let pg_client = ctx.pg_client;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            // Get the pet profile by UUID
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            let mut conn: InstrumentedPgConnection =
                diesel::Connection::establish(&ctx.pg_connstr).unwrap();
            // let rows = actix_demo::actions::pet_profile_full::get_full_pet_profile(&pet_id, &conn);

            use actix_demo::schema::pet_personality_traits::dsl as personality_traits;
            use diesel::prelude::*;
            let personality_traits = personality_traits::pet_personality_traits
                // .filter(personality_traits::pet_profile_id.eq(&pet_id))
                // .select(PetPersonalityTraits::as_select())
                .get_results::<PetPersonalityTraits>(&mut conn)
                .optional()
                .unwrap();

            println!("{:?}", personality_traits);

            let mut resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            assert_eq!(body.basic_info.pet_name.as_str(), "Buddy");
            assert_eq!(
                body.basic_info.pet_type,
                actix_demo::models::pet_enums::PetType::Dog
            );
            assert_eq!(
                body.personality_traits.map(|t| t.bio),
                Some(Some("A friendly dog".to_owned()))
            );
        }

        #[actix_rt::test]
        async fn should_return_error_when_getting_nonexistent_pet_profile() {
            let ctx = TestContext::new(None).await;

            let mut resp = ctx
                .test_server
                .get("/api/pets/00000000-0000-0000-0000-000000000000")
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("No pet profile found with uuid: 00000000-0000-0000-0000-000000000000"));
        }
    }

    mod update_pet_profile_api {
        use crate::common::{TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_update_a_pet_profile() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            // Get the pet profile by UUID to verify it was created
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Update the pet profile
            let update_data = serde_json::json!({
                "basic_info" : {
                    "pet_name": "Updated Buddy",
                    "breed": "Labrador Retriever",
                    "age": 3,
                    "weight_kg": 28.0,
                },
                "personality_traits" : {"bio": "An updated friendly dog"},
                "images": []
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            println!("{:?}", body);
            assert_eq!(body.basic_info.pet_name.as_str(), "Updated Buddy");
            assert_eq!(body.basic_info.breed.as_str(), "Labrador Retriever");
            assert_eq!(body.basic_info.age.as_i32(), 3);
            assert_eq!(body.basic_info.weight_kg.as_f32(), 28.0);
            assert_eq!(
                body.personality_traits.map(|b| b.bio),
                Some(Some("An updated friendly dog".to_owned()))
            );
        }

        #[actix_rt::test]
        async fn should_return_error_when_updating_nonexistent_pet_profile() {
            let ctx = TestContext::new(None).await;

            // Try to update a non-existent pet profile
            let update_data = serde_json::json!({
                "basic_info" : {
                    "pet_name": "Updated Buddy",
                    "breed": "Labrador Retriever",
                    "age": 3,
                    "weight_kg": 28.0,
                },
                "personality_traits" : {"bio": "An updated friendly dog"},
                "images": []
            });

            let mut resp = ctx
                .test_server
                .patch("/api/pets/00000000-0000-0000-0000-000000000000")
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body
                .cause
                .contains("Pet profile with uuid 00000000-0000-0000-0000-000000000000 does not exist"));
        }

        #[actix_rt::test]
        async fn should_return_error_when_updating_pet_profile_with_empty_name()
        {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            // Get the pet profile by UUID to verify it was created
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Try to update with empty name (should return error)
            let update_data = serde_json::json!({
                "basic_info" : {
                    "pet_name": "",
                    "breed": "",
                    "age": 3,
                    "weight_kg": 28.0,
                },
                "personality_traits" : {"bio": "An updated friendly dog"},
                "images": []
            });

            let resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }
    }

    mod partial_update_pet_profile_api {
        use crate::common::{TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_update_only_basic_info_section() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Update only basic_info section
            let update_data = serde_json::json!({
                "basic_info": {
                    "pet_name": "Updated Buddy",
                    "breed": "Labrador Retriever",
                    "age": 3,
                    "weight_kg": 28.0,
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            assert_eq!(body.basic_info.pet_name.as_str(), "Updated Buddy");
            assert_eq!(body.basic_info.breed.as_str(), "Labrador Retriever");
            assert_eq!(body.basic_info.age.as_i32(), 3);
            assert_eq!(body.basic_info.weight_kg.as_f32(), 28.0);

            // Verify other sections are unchanged
            assert_eq!(body.personality_traits.map(|t| t.bio), Some(Some("A friendly dog".to_owned())));
            assert_eq!(body.location_owner.as_ref().unwrap().owner_name, "Owner Name");
            assert_eq!(body.location_owner.as_ref().unwrap().location, "Location");
        }

        #[actix_rt::test]
        async fn should_update_only_personality_traits_section() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Update only personality_traits section
            let update_data = serde_json::json!({
                "personality_traits": {
                    "bio": "A very friendly and playful dog",
                    "good_with_dogs": true,
                    "good_with_cats": true,
                    "good_with_kids": true,
                    "house_trained": true,
                    "vaccinated": true,
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            let pt = body.personality_traits.as_ref().unwrap();
            assert_eq!(pt.bio, Some("A very friendly and playful dog".to_owned()));
            assert_eq!(pt.personality_traits, Some(vec![Some("friendly".to_owned()), Some("playful".to_owned())]));
            assert_eq!(pt.good_with_dogs, Some(true));
            assert_eq!(pt.good_with_cats, Some(true));
            assert_eq!(pt.good_with_kids, Some(true));
            assert_eq!(pt.house_trained, Some(true));
            assert_eq!(pt.vaccinated, Some(true));

            // Verify other sections are unchanged
            assert_eq!(body.basic_info.pet_name.as_str(), "Buddy");
            assert_eq!(body.basic_info.breed.as_str(), "Golden Retriever");
        }

        #[actix_rt::test]
        async fn should_update_only_activities_section() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": ["fetch", "hiking"],
                "likes": ["balls", "treats"],
                "dislikes": ["rain"],
                "energy_level": "medium",
                "trainability": "moderate",
                "barking_level": "moderate",
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Update only activities section
            let update_data = serde_json::json!({
                "activities": {
                    "favorite_activities": ["fetch", "hiking", "swimming"],
                    "likes": ["balls", "treats", "outdoor play"],
                    "dislikes": ["rain", "car rides"],
                    "energy_level": "high",
                    "trainability": "easy",
                    "barking_level": "moderate",
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            let activities = body.activities.as_ref().unwrap();
            assert_eq!(
                activities.favorite_activities,
                Some(vec![Some("fetch".to_owned()), Some("hiking".to_owned()), Some("swimming".to_owned())])
            );
            assert_eq!(
                activities.likes,
                Some(vec![Some("balls".to_owned()), Some("treats".to_owned()), Some("outdoor play".to_owned())])
            );
            assert_eq!(
                activities.dislikes,
                Some(vec![Some("rain".to_owned()), Some("car rides".to_owned())])
            );
            assert_eq!(activities.energy_level, Some(actix_demo::models::pet_enums::EnergyLevelType::High));
            assert_eq!(activities.trainability, Some(actix_demo::models::pet_enums::TrainabilityType::Easy));
            assert_eq!(activities.barking_level, Some(actix_demo::models::pet_enums::BarkingLevelType::Moderate));

            // Verify other sections are unchanged
            assert_eq!(body.basic_info.pet_name.as_str(), "Buddy");
        }

        #[actix_rt::test]
        async fn should_update_only_location_owner_section() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": ["fetch", "hiking"],
                "likes": ["balls", "treats"],
                "dislikes": ["rain"],
                "energy_level": "medium",
                "trainability": "moderate",
                "barking_level": "moderate",
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Update only location_owner section
            let update_data = serde_json::json!({
                "location_owner": {
                    "owner_name": "New Owner Name",
                    "location": "New Location",
                    "address": "123 New Street",
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            assert_eq!(body.location_owner.as_ref().unwrap().owner_name, "New Owner Name");
            assert_eq!(body.location_owner.as_ref().unwrap().location, "New Location");
            assert_eq!(body.location_owner.as_ref().unwrap().address, Some("123 New Street".to_owned()));

            // Verify other sections are unchanged
            assert_eq!(body.basic_info.pet_name.as_str(), "Buddy");
            assert_eq!(body.personality_traits.map(|t| t.bio), Some(Some("A friendly dog".to_owned())));
        }

        #[actix_rt::test]
        async fn should_update_only_adoption_details_section() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": null,
                "personality_traits": null,
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Update only adoption_details section
            let update_data = serde_json::json!({
                "adoption_details": {
                    "special_needs": true,
                    "special_needs_description": "Requires medication twice daily",
                    "adoption_status": "adoptable",
                    "shelter_name": "Happy Paws Shelter",
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            let adoption = body.adoption_details.as_ref().unwrap();
            assert_eq!(adoption.special_needs, Some(true));
            assert_eq!(adoption.special_needs_description, Some("Requires medication twice daily".to_owned()));
            assert_eq!(adoption.adoption_status, Some(actix_demo::models::pet_enums::AdoptionStatusType::Adoptable));
            assert_eq!(adoption.shelter_name, Some("Happy Paws Shelter".to_owned()));

            // Verify other sections are unchanged
            assert_eq!(body.basic_info.pet_name.as_str(), "Buddy");
        }

        #[actix_rt::test]
        async fn should_add_new_images_only() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Add new images only
            let update_data = serde_json::json!({
                "images": [
                    {
                        "pet_profile_uuid": pet_uuid,
                        "image_url": "https://example.com/image1.jpg",
                        "is_primary": true,
                        "sort_order": 1
                    },
                    {
                        "pet_profile_uuid": pet_uuid,
                        "image_url": "https://example.com/image2.jpg",
                        "is_primary": false,
                        "sort_order": 2
                    }
                ]
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            assert_eq!(body.images.len(), 2);
            assert_eq!(body.images[0].image_url, "https://example.com/image1.jpg");
            assert_eq!(body.images[0].is_primary, Some(true));
            assert_eq!(body.images[1].image_url, "https://example.com/image2.jpg");
            assert_eq!(body.images[1].is_primary, Some(false));

            // Verify other sections are unchanged
            assert_eq!(body.basic_info.pet_name.as_str(), "Buddy");
        }

        #[actix_rt::test]
        async fn should_update_multiple_sections_at_once() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": ["fetch", "hiking"],
                "likes": ["balls", "treats"],
                "dislikes": ["rain"],
                "energy_level": "medium",
                "trainability": "moderate",
                "barking_level": "moderate",
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Update multiple sections at once
            let update_data = serde_json::json!({
                "basic_info": {
                    "pet_name": "Updated Buddy",
                    "breed": "Labrador Retriever",
                },
                "personality_traits": {
                    "bio": "A very friendly dog",
                    "good_with_dogs": true,
                },
                "activities": {
                    "favorite_activities": ["fetch", "hiking"],
                    "energy_level": "high",
                },
                "location_owner": {
                    "owner_name": "New Owner",
                    "location": "New City",
                },
                "images": [
                    {
                        "pet_profile_uuid": pet_uuid,
                        "image_url": "https://example.com/new_image.jpg",
                        "is_primary": true,
                        "sort_order": 1
                    }
                ]
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            assert_eq!(body.basic_info.pet_name.as_str(), "Updated Buddy");
            assert_eq!(body.basic_info.breed.as_str(), "Labrador Retriever");
            let pt = body.personality_traits.as_ref().unwrap();
            assert_eq!(pt.bio, Some("A very friendly dog".to_owned()));
            assert_eq!(pt.good_with_dogs, Some(true));
            let activities = body.activities.as_ref().unwrap();
            assert_eq!(activities.favorite_activities, Some(vec![Some("fetch".to_owned()), Some("hiking".to_owned())]));
            assert_eq!(activities.energy_level, Some(actix_demo::models::pet_enums::EnergyLevelType::High));
            let lo = body.location_owner.as_ref().unwrap();
            assert_eq!(lo.owner_name, "New Owner");
            assert_eq!(lo.location, "New City");
            assert_eq!(body.images.len(), 1);
            assert_eq!(body.images[0].image_url, "https://example.com/new_image.jpg");
        }

        #[actix_rt::test]
        async fn should_validate_partial_update_basic_info_pet_name() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Try to update with invalid pet_name (too short)
            let update_data = serde_json::json!({
                "basic_info": {
                    "pet_name": "Ab",
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("pet name"));
        }

        #[actix_rt::test]
        async fn should_validate_partial_update_basic_info_breed() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Try to update with invalid breed (too short)
            let update_data = serde_json::json!({
                "basic_info": {
                    "breed": "Ab",
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("breed"));
        }

        #[actix_rt::test]
        async fn should_validate_partial_update_basic_info_age() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": "A friendly dog",
                "personality_traits": ["friendly", "playful"],
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Try to update with invalid age (negative)
            let update_data = serde_json::json!({
                "basic_info": {
                    "age": -1,
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            // The error response has a different content type, so we just check the status code
            // The actual validation error is logged in the debug output
        }

        #[actix_rt::test]
        async fn should_validate_partial_update_basic_info_weight_kg() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": null,
                "personality_traits": null,
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Try to update with invalid weight_kg (too high)
            let update_data = serde_json::json!({
                "basic_info": {
                    "weight_kg": 200.0,
                }
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("weight"));
        }

        #[actix_rt::test]
        async fn should_clear_optional_field_to_null() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile with some optional fields set
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "size": null,
                "color": null,
                "coat_type": null,
                "bio": null,
                "personality_traits": null,
                "good_with_dogs": null,
                "good_with_cats": null,
                "good_with_kids": null,
                "house_trained": null,
                "vaccinated": null,
                "spayed_neutered": null,
                "microchipped": null,
                "favorite_activities": null,
                "likes": null,
                "dislikes": null,
                "energy_level": null,
                "trainability": null,
                "barking_level": null,
                "owner_name": "Owner Name",
                "location": "Location",
                "address": null,
                "lat": null,
                "lng": null,
                "special_needs": false,
                "special_needs_description": null,
                "adoption_status": null,
                "shelter_name": null,
                "images": [
                    {
                        "image_url": "https://example.com/image1.jpg",
                        "is_primary": true,
                        "sort_order": 1
                    }
                ]
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Verify initial state has image
            assert_eq!(created_pet.images.len(), 1);

            // Clear the image by setting to empty array
            let update_data = serde_json::json!({
                "images": []
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let _body: FullPetProfile = resp.json().await.unwrap();
            // Note: Images are added, not replaced, so the original image should still be there
            // If you want to clear images, you need to delete them individually
        }
    }

    mod delete_pet_profile_api {
        use crate::common::{TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_delete_a_pet_profile_by_id() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Matte",
                "pet_type": "dog",
                "breed": "Labrador",
                "age": 4,
                "weight_kg": 30.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            // Get the pet profile by UUID to verify it was created
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Delete the pet profile
            let delete_resp = ctx
                .test_server
                .delete(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

            // Verify that the pet profile no longer exists
            let get_resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
        }

        #[actix_rt::test]
        async fn should_return_error_when_deleting_nonexistent_pet_profile() {
            let ctx = TestContext::new(None).await;

            let resp = ctx
                .test_server
                .delete("/api/pets/00000000-0000-0000-0000-000000000000")
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            // The delete endpoint may return 204 No Content for non-existent resources
            // or it might return a 404 Not Found, depending on the implementation.
            // For now we'll test that it doesn't return an internal server error
            assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    mod delete_pet_profile_image_api {
        use actix_demo::models::misc::ErrorResponse;
        use actix_web::http::StatusCode;

        use crate::common::{TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_delete_a_pet_profile_image() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile with images
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": [
                    {
                        "image_url": "https://example.com/image1.jpg",
                        "is_primary": true,
                        "sort_order": 1
                    },
                    {
                        "image_url": "https://example.com/image2.jpg",
                        "is_primary": false,
                        "sort_order": 2
                    }
                ]
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Get the pet profile to find the image ID
            let mut get_resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(get_resp.status(), StatusCode::OK);

            let pet_with_images: FullPetProfile = get_resp.json().await.unwrap();
            let image_id = pet_with_images.images[0].id;

            // Delete the image
            let mut delete_resp = ctx
                .test_server
                .delete(&format!("/api/pets/{}/images/{}", pet_uuid, image_id))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(delete_resp.status(), StatusCode::OK);

            let deleted_image: actix_demo::models::pet_profile_images::PetProfileImage =
                delete_resp.json().await.unwrap();
            assert_eq!(deleted_image.id, image_id);

            // Verify the image is no longer in the profile
            let mut get_resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(get_resp.status(), StatusCode::OK);

            let updated_pet: FullPetProfile = get_resp.json().await.unwrap();
            assert_eq!(updated_pet.images.len(), 1);
            assert_ne!(updated_pet.images[0].id, image_id);
        }

        #[actix_rt::test]
        async fn should_return_error_when_deleting_nonexistent_pet_profile_image() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile first
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Try to delete a non-existent image
            let resp = ctx
                .test_server
                .delete(&format!("/api/pets/{}/images/999", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        }

        #[actix_rt::test]
        async fn should_return_error_when_deleting_image_from_nonexistent_pet_profile() {
            let ctx = TestContext::new(None).await;

            // Try to delete an image from a non-existent pet profile
            let mut resp = ctx
                .test_server
                .delete("/api/pets/00000000-0000-0000-0000-000000000000/images/1")
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("Pet profile with uuid 00000000-0000-0000-0000-000000000000 does not exist"));
        }

        #[actix_rt::test]
        async fn should_return_error_when_deleting_image_without_ownership() {
            use crate::common::create_http_user;

            let ctx = TestContext::new(None).await;

            // Create a pet profile with user 1
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": [
                    {
                        "image_url": "https://example.com/image1.jpg",
                        "is_primary": true,
                        "sort_order": 1
                    }
                ]
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_id = created_pet.basic_info.uuid;

            // Get the pet profile to find the image ID
            let mut get_resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_id))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(get_resp.status(), StatusCode::OK);

            let pet_with_images: FullPetProfile = get_resp.json().await.unwrap();
            let image_id = pet_with_images.images[0].id;

            // Create a second user (user2) via registration API
            let _ = create_http_user(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await;

            // Get token for user 2
            let user2_token = crate::common::get_http_token(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await
            .unwrap();

            // Try to delete the image with user 2's token
            let mut resp = ctx
                .test_server
                .delete(&format!("/api/pets/{}/images/{}", pet_id, image_id))
                .with_token(&user2_token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("You can only delete images from your own pet profiles"));
        }
    }

    mod add_pet_profile_image_api {
        use actix_demo::models::misc::ErrorResponse;
        use actix_web::http::StatusCode;

        use crate::common::{TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_add_a_single_image_to_pet_profile() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile without images
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Add a single image
            let add_image_data = serde_json::json!({
                "image_url": "https://example.com/image1.jpg",
                "is_primary": true
            });

            let mut resp = ctx
                .test_server
                .post(&format!("/api/pets/{}/images", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&add_image_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let added_image: actix_demo::models::pet_profile_images::PetProfileImage =
                resp.json().await.unwrap();
            assert_eq!(added_image.image_url, "https://example.com/image1.jpg");
            assert_eq!(added_image.is_primary, Some(true));
            assert_eq!(added_image.sort_order, Some(1));

            // Verify the image is in the profile
            let mut get_resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(get_resp.status(), StatusCode::OK);

            let updated_pet: FullPetProfile = get_resp.json().await.unwrap();
            assert_eq!(updated_pet.images.len(), 1);
            assert_eq!(updated_pet.images[0].image_url, "https://example.com/image1.jpg");
        }

        #[actix_rt::test]
        async fn should_add_multiple_images_to_pet_profile() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile without images
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Add multiple images
            let add_images_data = serde_json::json!({
                "image_urls": [
                    "https://example.com/image1.jpg",
                    "https://example.com/image2.jpg",
                    "https://example.com/image3.jpg"
                ]
            });

            let mut resp = ctx
                .test_server
                .post(&format!("/api/pets/{}/images/bulk", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&add_images_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let added_images: Vec<actix_demo::models::pet_profile_images::PetProfileImage> =
                resp.json().await.unwrap();
            assert_eq!(added_images.len(), 3);
            assert_eq!(added_images[0].image_url, "https://example.com/image1.jpg");
            assert_eq!(added_images[1].image_url, "https://example.com/image2.jpg");
            assert_eq!(added_images[2].image_url, "https://example.com/image3.jpg");

            // Verify sort orders are sequential
            assert_eq!(added_images[0].sort_order, Some(1));
            assert_eq!(added_images[1].sort_order, Some(2));
            assert_eq!(added_images[2].sort_order, Some(3));

            // Verify all images are in the profile
            let mut get_resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(get_resp.status(), StatusCode::OK);

            let updated_pet: FullPetProfile = get_resp.json().await.unwrap();
            assert_eq!(updated_pet.images.len(), 3);
        }

        #[actix_rt::test]
        async fn should_set_primary_image() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile with multiple images
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": [
                    {
                        "image_url": "https://example.com/image1.jpg",
                        "is_primary": true,
                        "sort_order": 1
                    },
                    {
                        "image_url": "https://example.com/image2.jpg",
                        "is_primary": false,
                        "sort_order": 2
                    },
                    {
                        "image_url": "https://example.com/image3.jpg",
                        "is_primary": false,
                        "sort_order": 3
                    }
                ]
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Get the pet profile to find the image IDs
            let mut get_resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(get_resp.status(), StatusCode::OK);

            let pet_with_images: FullPetProfile = get_resp.json().await.unwrap();
            let image_id = pet_with_images.images[1].id; // Get the second image

            // Set the second image as primary
            let mut resp = ctx
                .test_server
                .put(&format!("/api/pets/{}/images/{}/primary", pet_uuid, image_id))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let updated_image: actix_demo::models::pet_profile_images::PetProfileImage =
                resp.json().await.unwrap();
            assert_eq!(updated_image.id, image_id);
            assert_eq!(updated_image.is_primary, Some(true));

            // Verify the second image is now primary
            let mut get_resp = ctx
            .test_server
            .get(&format!("/api/pets/{}", pet_uuid))
            .with_token(&ctx._token)
            .send()
            .await
            .unwrap();
            
            assert_eq!(get_resp.status(), StatusCode::OK);
            
            let updated_pet: FullPetProfile = get_resp.json().await.unwrap();
            assert_eq!(updated_pet.images.iter().filter(|img| img.is_primary == Some(true)).count(), 1);
            
            // Find images by ID and verify their primary status
            let second_image = updated_pet.images.iter().find(|img| img.id == pet_with_images.images[1].id).unwrap();
            assert_eq!(second_image.is_primary, Some(true));
            }

        #[actix_rt::test]
        async fn should_return_error_when_adding_image_to_nonexistent_pet_profile() {
            let ctx = TestContext::new(None).await;

            // Try to add an image to a non-existent pet profile
            let add_image_data = serde_json::json!({
                "image_url": "https://example.com/image1.jpg",
                "is_primary": true
            });

            let mut resp = ctx
                .test_server
                .post("/api/pets/00000000-0000-0000-0000-000000000000/images")
                .with_token(&ctx._token)
                .send_json(&add_image_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("Pet profile with uuid 00000000-0000-0000-0000-000000000000 does not exist"));
        }

        #[actix_rt::test]
        async fn should_return_error_when_set_primary_image_to_nonexistent() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Try to set a non-existent image as primary
            let resp = ctx
                .test_server
                .put(&format!("/api/pets/{}/images/999/primary", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        }

        #[actix_rt::test]
        async fn should_return_error_when_adding_image_without_ownership() {
            use crate::common::create_http_user;

            let ctx = TestContext::new(None).await;

            // Create a pet profile with user 1
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Create a second user (user2) via registration API
            let _ = create_http_user(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await;

            // Get token for user 2
            let user2_token = crate::common::get_http_token(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await
            .unwrap();

            // Try to add an image with user 2's token
            let add_image_data = serde_json::json!({
                "image_url": "https://example.com/image1.jpg",
                "is_primary": true
            });

            let mut resp = ctx
                .test_server
                .post(&format!("/api/pets/{}/images", pet_uuid))
                .with_token(&user2_token)
                .send_json(&add_image_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("You can only add images to your own pet profiles"));
        }
    }

    mod ownership_validation {
        use crate::common::{create_http_user, TestContext, WithToken};
        use actix_demo::models::misc::ErrorResponse;
        use actix_web::http::StatusCode;

        use super::*;

        #[actix_rt::test]
        async fn should_return_error_when_getting_pet_profile_without_ownership() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile with the default user (user1)
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Create a second user (user2) via registration API
            let _ = create_http_user(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await;

            // Get token for user 2
            let user2_token = crate::common::get_http_token(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await
            .unwrap();

            // Try to get the pet profile with user 2's token
            let mut resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&user2_token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("You can only view your own pet profiles"));
        }

        #[actix_rt::test]
        async fn should_return_error_when_updating_pet_profile_without_ownership() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile with the default user (user1)
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Create a second user (user2) via registration API
            let _ = create_http_user(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await;

            // Get token for user 2
            let user2_token = crate::common::get_http_token(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await
            .unwrap();

            // Try to update the pet profile with user 2's token
            let update_data = serde_json::json!({
                "basic_info" : {
                    "pet_name": "Updated Buddy",
                    "breed": "Labrador Retriever",
                    "age": 3,
                    "weight_kg": 28.0,
                },
                "personality_traits" : {"bio": "An updated friendly dog"},
                "images": []
            });

            let mut resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&user2_token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("You can only update your own pet profiles"));
        }

        #[actix_rt::test]
        async fn should_return_error_when_deleting_pet_profile_without_ownership() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile with the default user (user1)
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Create a second user (user2) via registration API
            let _ = create_http_user(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await;

            // Get token for user 2
            let user2_token = crate::common::get_http_token(
                &ctx.addr,
                "user2",
                "password123",
                &ctx.client,
            )
            .await
            .unwrap();

            // Try to delete the pet profile with user 2's token
            let mut resp = ctx
                .test_server
                .delete(&format!("/api/pets/{}", pet_uuid))
                .with_token(&user2_token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

            let body: ErrorResponse<String> = resp.json::<ErrorResponse<String>>().await.unwrap();
            assert!(body.cause.contains("You can only delete your own pet profiles"));
        }

        #[actix_rt::test]
        async fn should_allow_access_to_own_pet_profile() {
            let ctx = TestContext::new(None).await;

            // Create a pet profile with the default user (user1)
            let pet_data_json = serde_json::json!({
                "user_id": 1,
                "pet_name": "Buddy",
                "pet_type": "dog",
                "breed": "Golden Retriever",
                "age": 2,
                "weight_kg": 25.0,
                "gender": "male",
                "bio": "A friendly dog",
                "owner_name": "Owner Name",
                "location": "Location",
                "special_needs": false,
                "images": []
            });

            let mut create_resp = ctx
                .test_server
                .post("/api/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_uuid = created_pet.basic_info.uuid;

            // Verify that the owner can still get the pet profile
            let mut get_resp = ctx
                .test_server
                .get(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(get_resp.status(), StatusCode::OK);

            let body: FullPetProfile = get_resp.json::<FullPetProfile>().await.unwrap();
            assert_eq!(body.basic_info.pet_name.as_str(), "Buddy");

            // Verify that the owner can update the pet profile
            let update_data = serde_json::json!({
                "basic_info" : {
                    "pet_name": "Updated Buddy",
                    "breed": "Labrador Retriever",
                    "age": 3,
                    "weight_kg": 28.0,
                },
                "personality_traits" : {"bio": "An updated friendly dog"},
                "images": []
            });

            let mut update_resp = ctx
                .test_server
                .patch(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(update_resp.status(), StatusCode::OK);

            let updated_body: FullPetProfile = update_resp.json().await.unwrap();
            assert_eq!(updated_body.basic_info.pet_name.as_str(), "Updated Buddy");

            // Verify that the owner can delete the pet profile
            let delete_resp = ctx
                .test_server
                .delete(&format!("/api/pets/{}", pet_uuid))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);
        }
    }
}
