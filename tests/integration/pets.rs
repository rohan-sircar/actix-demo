mod pet_profiles_api {
    use actix_web::http::header::CONTENT_TYPE;
    use actix_web::http::StatusCode;

    use crate::common::{register_and_login, TestContext, WithToken};

    #[actix_rt::test]
    async fn create_pet_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner1", "test123").await;

        let mut resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog",
                "breed": "Golden Retriever",
                "date_of_birth": "2020-05-15",
                "gender": "male",
                "weight": 30.5,
                "color_markings": "Golden",
                "description": "A friendly dog",
                "traits": ["playful", "loyal"]
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["name"], "Buddy");
        assert_eq!(body["species"], "dog");
        assert_eq!(body["breed"], "Golden Retriever");
        assert_eq!(body["date_of_birth"], "2020-05-15");
        assert_eq!(body["gender"], "male");
        assert_eq!(body["weight"], 30.5);
        assert_eq!(body["color_markings"], "Golden");
        assert_eq!(body["description"], "A friendly dog");
        assert_eq!(body["traits"].as_array().unwrap().len(), 2);
        assert_eq!(body["traits"][0]["name"], "playful");
        assert_eq!(body["traits"][1]["name"], "loyal");

        let pet_uuid = body["pet_uuid"].as_str().unwrap().to_string();

        // Verify the pet was created in the database
        let get_resp = ctx
            .test_server
            .get(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(get_resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn create_pet_partial_fields() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner2", "test123").await;

        let mut resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Whiskers",
                "species": "cat"
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["name"], "Whiskers");
        assert_eq!(body["species"], "cat");
        assert!(body["breed"].is_null());
        assert!(body["traits"].as_array().unwrap().is_empty());
    }

    #[actix_rt::test]
    async fn create_pet_with_invalid_weight() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner3", "test123").await;

        let resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog",
                "weight": -5.0
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn create_pet_with_nonexistent_trait() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner4", "test123").await;

        let mut resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog",
                "traits": ["playful", "nonexistent_trait"]
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value = resp.json().await.unwrap();
        // Only the valid trait should be assigned
        assert_eq!(body["traits"].as_array().unwrap().len(), 1);
        assert_eq!(body["traits"][0]["name"], "playful");
    }

    #[actix_rt::test]
    async fn get_pet_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner5", "test123").await;

        // Create a pet first
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Max",
                "species": "dog",
                "traits": ["energetic"]
            }))
            .await
            .unwrap();
        let pet_uuid = create_resp.json::<serde_json::Value>().await.unwrap()
            ["pet_uuid"]
            .as_str()
            .unwrap()
            .to_string();

        // Get the pet
        let mut resp = ctx
            .test_server
            .get(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["name"], "Max");
        assert_eq!(body["species"], "dog");
        assert_eq!(body["traits"].as_array().unwrap().len(), 1);
    }

    #[actix_rt::test]
    async fn get_pet_not_found() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner6", "test123").await;

        let resp = ctx
            .test_server
            .get("/api/v1/private/user/pets/00000000-0000-0000-0000-000000000000")
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn get_pet_ownership_enforcement() {
        let ctx = TestContext::new(None).await;
        let token1 = register_and_login(&ctx, "owner_a", "test123").await;
        let token2 = register_and_login(&ctx, "owner_b", "test123").await;

        // Owner A creates a pet
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token1)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog"
            }))
            .await
            .unwrap();
        let pet_uuid = create_resp.json::<serde_json::Value>().await.unwrap()
            ["pet_uuid"]
            .as_str()
            .unwrap()
            .to_string();

        // Owner B tries to get the pet
        let resp = ctx
            .test_server
            .get(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token2)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn list_pets_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner7", "test123").await;

        // Create multiple pets
        ctx.test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog",
                "traits": ["playful"]
            }))
            .await
            .unwrap();

        ctx.test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Whiskers",
                "species": "cat",
                "traits": ["calm"]
            }))
            .await
            .unwrap();

        ctx.test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Rex",
                "species": "dog",
                "traits": ["protective"]
            }))
            .await
            .unwrap();

        // List all pets
        let mut resp = ctx
            .test_server
            .get("/api/v1/private/user/pets")
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert_eq!(body.len(), 3);

        // Filter by species
        let mut resp = ctx
            .test_server
            .get("/api/v1/private/user/pets?species=dog")
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert_eq!(body.len(), 2);
        assert!(body.iter().all(|p| p["species"] == "dog"));

        // Filter by traits
        let mut resp = ctx
            .test_server
            .get("/api/v1/private/user/pets?traits=playful")
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert_eq!(body.len(), 1);
        assert_eq!(body[0]["name"], "Buddy");
    }

    #[actix_rt::test]
    async fn update_pet_partial_update() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner8", "test123").await;

        // Create a pet
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog",
                "breed": "Golden Retriever",
                "weight": 30.5
            }))
            .await
            .unwrap();
        let pet_uuid = create_resp.json::<serde_json::Value>().await.unwrap()
            ["pet_uuid"]
            .as_str()
            .unwrap()
            .to_string();

        // Partial update - only change name and weight
        let mut resp = ctx
            .test_server
            .patch(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Max",
                "weight": 35.0
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["name"], "Max");
        assert_eq!(body["weight"], 35.0);
        assert_eq!(body["breed"], "Golden Retriever"); // unchanged
    }

    #[actix_rt::test]
    async fn update_pet_clear_field() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner9", "test123").await;

        // Create a pet with breed
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog",
                "breed": "Golden Retriever"
            }))
            .await
            .unwrap();
        let pet_uuid = create_resp.json::<serde_json::Value>().await.unwrap()
            ["pet_uuid"]
            .as_str()
            .unwrap()
            .to_string();

        // Clear the breed field
        let mut resp = ctx
            .test_server
            .patch(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "breed": null
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["breed"].is_null());
    }

    #[actix_rt::test]
    async fn update_pet_replaces_traits() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner10", "test123").await;

        // Create a pet with traits
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog",
                "traits": ["playful", "loyal"]
            }))
            .await
            .unwrap();
        let pet_uuid = create_resp.json::<serde_json::Value>().await.unwrap()
            ["pet_uuid"]
            .as_str()
            .unwrap()
            .to_string();

        // Replace traits
        let mut resp = ctx
            .test_server
            .patch(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "traits": ["calm", "independent"]
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        let trait_names: Vec<&str> = body["traits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(trait_names.contains(&"calm"));
        assert!(trait_names.contains(&"independent"));
        assert!(!trait_names.contains(&"playful"));
        assert!(!trait_names.contains(&"loyal"));
    }

    #[actix_rt::test]
    async fn delete_pet_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner11", "test123").await;

        // Create a pet
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog"
            }))
            .await
            .unwrap();
        let pet_uuid = create_resp.json::<serde_json::Value>().await.unwrap()
            ["pet_uuid"]
            .as_str()
            .unwrap()
            .to_string();

        // Delete the pet
        let resp = ctx
            .test_server
            .delete(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify it's gone
        let resp = ctx
            .test_server
            .get(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn delete_pet_ownership_enforcement() {
        let ctx = TestContext::new(None).await;
        let token1 = register_and_login(&ctx, "owner_x", "test123").await;
        let token2 = register_and_login(&ctx, "owner_y", "test123").await;

        // Owner X creates a pet
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token1)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog"
            }))
            .await
            .unwrap();
        let pet_uuid = create_resp.json::<serde_json::Value>().await.unwrap()
            ["pet_uuid"]
            .as_str()
            .unwrap()
            .to_string();

        // Owner Y tries to delete the pet
        let resp = ctx
            .test_server
            .delete(format!("/api/v1/private/user/pets/{}", pet_uuid))
            .with_token(&token2)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn get_traits_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "owner_x", "test123").await;

        let mut resp = ctx
            .test_server
            .get("/api/v1/private/pets/traits")
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert!(!body.is_empty());

        // Check that expected traits exist
        let trait_names: Vec<&str> =
            body.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(trait_names.contains(&"playful"));
        assert!(trait_names.contains(&"calm"));
        assert!(trait_names.contains(&"energetic"));
        assert!(trait_names.contains(&"smart"));
    }

    #[actix_rt::test]
    async fn get_public_pet_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner12", "test123").await;

        // Create a pet
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog",
                "breed": "Labrador",
                "traits": ["playful"]
            }))
            .await
            .unwrap();
        let pet_uuid = create_resp.json::<serde_json::Value>().await.unwrap()
            ["pet_uuid"]
            .as_str()
            .unwrap()
            .to_string();

        let mut resp = ctx
            .test_server
            .get(format!("/api/v1/private/pets/{}", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["name"], "Buddy");
        assert_eq!(body["species"], "dog");
        assert_eq!(body["breed"], "Labrador");
        assert_eq!(body["traits"].as_array().unwrap().len(), 1);
    }

    #[actix_rt::test]
    async fn public_pet_returns_404_for_nonexistent() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner12", "test123").await;

        let resp = ctx
            .test_server
            .get("/api/v1/private/pets/00000000-0000-0000-0000-000000000000")
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn list_pets_returns_empty_for_user_with_no_pets() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "petowner_empty", "test123").await;

        let mut resp = ctx
            .test_server
            .get("/api/v1/private/user/pets")
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert!(body.is_empty());
    }

    #[actix_rt::test]
    async fn public_pet_returns_owner_info() {
        let ctx = TestContext::new(None).await;

        let token = register_and_login(&ctx, "ownerinfo_user", "test123").await;

        // Create a profile for the user
        let profile_resp = ctx
            .test_server
            .post("/api/v1/private/user/profile")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "display_name": "Test Owner"
            }))
            .await
            .unwrap();
        assert_eq!(profile_resp.status(), StatusCode::CREATED);

        // Create a pet
        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog"
            }))
            .await
            .unwrap();
        let pet = create_resp.json::<serde_json::Value>().await.unwrap();
        let pet_uuid = pet["pet_uuid"].as_str().unwrap();

        // Fetch the public pet
        let mut resp = ctx
            .test_server
            .get(format!("/api/v1/private/pets/{}", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();

        // Verify owner info is present
        assert!(
            body["owner"].is_object(),
            "owner should be an object, got: {}",
            body["owner"]
        );
        let owner = &body["owner"];
        assert!(
            owner["user_uuid"].as_str().is_some(),
            "owner should have user_uuid"
        );
        assert!(
            owner["pets_owned"].as_u64().is_some(),
            "owner should have pets_owned"
        );
        assert_eq!(
            owner["pets_owned"].as_u64(),
            Some(1),
            "owner should have 1 pet"
        );
    }
}
