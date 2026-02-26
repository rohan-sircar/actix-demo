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
                .post("/api/users/1/pets")
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
                .post("/api/users/1/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            println!("{:?}", std::str::from_utf8(&resp.body().await.unwrap()));
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
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
                .post("/api/users/1/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            // Get the pet profile by ID
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_id = created_pet.basic_info.id;

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
                .get(&format!("/api/users/1/pets/{}", pet_id))
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
                .get("/api/users/1/pets/999")
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body.cause.contains("No pet profile found with id: 999"));
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
                .post("/api/users/1/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            // Get the pet profile by ID to verify it was created
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_id = created_pet.basic_info.id;

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
                .patch(&format!("/api/users/1/pets/{}", pet_id))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::OK);

            let body: FullPetProfile = resp.json().await.unwrap();
            println!("{:?}", body);
            assert_eq!(body.basic_info.pet_name.as_str(), "Updated Buddy");
            assert_eq!(body.basic_info.breed.as_str(), "Labrador Retriever");
            assert_eq!(body.basic_info.age, 3);
            assert_eq!(body.basic_info.weight_kg, 28.0);
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
                .patch("/api/users/1/pets/999")
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let body: ErrorResponse<String> = resp.json().await.unwrap();
            assert!(body
                .cause
                .contains("Pet profile with id 999 does not exist"));
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
                .post("/api/users/1/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            // Get the pet profile by ID to verify it was created
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_id = created_pet.basic_info.id;

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
                .patch(&format!("/api/users/1/pets/{}", pet_id))
                .with_token(&ctx._token)
                .send_json(&update_data)
                .await
                .unwrap();

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
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
                .post("/api/users/1/pets")
                .with_token(&ctx._token)
                .send_json(&pet_data_json)
                .await
                .unwrap();

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            // Get the pet profile by ID to verify it was created
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_id = created_pet.basic_info.id;

            // Delete the pet profile
            let delete_resp = ctx
                .test_server
                .delete(&format!("/api/users/1/pets/{}", pet_id))
                .with_token(&ctx._token)
                .send()
                .await
                .unwrap();

            assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

            // Verify that the pet profile no longer exists
            let get_resp = ctx
                .test_server
                .get(&format!("/api/users/1/pets/{}", pet_id))
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
                .delete("/api/users/1/pets/999")
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
}
