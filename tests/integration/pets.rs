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

        use crate::common::{TestContext, WithToken};

        use super::*;

        #[actix_rt::test]
        async fn should_get_a_pet_profile_by_id() {
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

            // Get the pet profile by ID
            let created_pet: FullPetProfile = create_resp.json().await.unwrap();
            let pet_id = created_pet.basic_info.id;

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
}
