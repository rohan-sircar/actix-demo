mod discover {
    use actix_web::http::header::CONTENT_TYPE;
    use actix_web::http::StatusCode;

    use crate::common::{register_and_login, TestContext, WithToken};

    mod schema_test {
        use actix_demo::models::likes::{Like, LikeDirection, NewLike};
        use actix_demo::models::pets::{CreatePet, PetId, PetName, PetSpecies};
        use actix_demo::models::roles::RoleEnum;
        use actix_demo::models::users::{
            Email, NewUser, Password, UserId, Username,
        };
        use actix_demo::schema::likes as likes_schema;
        use diesel::prelude::*;
        use validators::prelude::*;

        use crate::common::TestContext;

        #[actix_rt::test]
        async fn likes_schema_verification() {
            let ctx = TestContext::new(None).await;

            let mut conn = ctx.app_data.pool.get().unwrap();

            let new_user1 = NewUser {
                username: Username::parse_str("schema_test_1").unwrap(),
                password: Password::parse_str("test123").unwrap(),
                email: Email::try_from("schema_test_1@test.local".to_string())
                    .unwrap(),
            };
            let new_user2 = NewUser {
                username: Username::parse_str("schema_test_2").unwrap(),
                password: Password::parse_str("test123").unwrap(),
                email: Email::try_from("schema_test_2@test.local".to_string())
                    .unwrap(),
            };

            let user1 = actix_demo::actions::users::insert_new_user(
                new_user1,
                RoleEnum::RoleUser,
                ctx.app_data.config.hash_cost,
                &ctx.app_data.user_ids_cache,
                &mut conn,
            )
            .unwrap();

            let user2 = actix_demo::actions::users::insert_new_user(
                new_user2,
                RoleEnum::RoleUser,
                ctx.app_data.config.hash_cost,
                &ctx.app_data.user_ids_cache,
                &mut conn,
            )
            .unwrap();

            let user1_uid = UserId::try_from(user1.id.as_uint()).unwrap();
            let user2_uid = UserId::try_from(user2.id.as_uint()).unwrap();

            let create_pet = CreatePet {
                name: PetName::new("TestDog".to_string()).unwrap(),
                species: PetSpecies::new("dog".to_string()).unwrap(),
                breed: None,
                date_of_birth: None,
                gender: None,
                weight: None,
                color_markings: None,
                description: None,
                traits: vec![],
            };

            let pet_result = actix_demo::actions::pets::create_pet(
                &user2_uid, create_pet, &mut conn,
            )
            .unwrap();

            let pet_id = PetId::try_from(pet_result.id.as_uint()).unwrap();

            let new_like =
                NewLike::new(user1_uid, user2_uid, pet_id, LikeDirection::Like);
            let result = diesel::insert_into(likes_schema::table)
                .values(&new_like)
                .execute(&mut conn);

            assert!(
                result.is_ok(),
                "Failed to insert into likes table: {:?}",
                result.err()
            );

            let liked_likes: Vec<Like> = likes_schema::table
                .order(likes_schema::id.asc())
                .load(&mut conn)
                .expect("Failed to query likes table");

            assert_eq!(liked_likes.len(), 1);
            let like = &liked_likes[0];
            assert_eq!(like.direction, LikeDirection::Like);
            assert_eq!(like.is_match, false);

            #[derive(diesel::deserialize::QueryableByName, Debug)]
            struct IndexInfo {
                #[diesel(sql_type = diesel::sql_types::Text)]
                name: String,
            }
            let index_rows: Vec<IndexInfo> = diesel::sql_query(
                "SELECT indexname as name FROM pg_indexes WHERE tablename = 'likes' ORDER BY indexname"
            )
            .load(&mut conn)
            .expect("Failed to query indexes");

            let index_names: Vec<&str> =
                index_rows.iter().map(|r| r.name.as_str()).collect();
            assert!(
                index_names
                    .iter()
                    .any(|n| n.contains("idx_likes_user_pet_owner")),
                "Missing idx_likes_user_pet_owner, found: {:?}",
                index_names
            );
            assert!(
                index_names
                    .iter()
                    .any(|n| n.contains("idx_likes_user_pet_unique")),
                "Missing idx_likes_user_pet_unique, found: {:?}",
                index_names
            );
            assert!(
                index_names.iter().any(|n| n.contains("idx_likes_is_match")),
                "Missing idx_likes_is_match, found: {:?}",
                index_names
            );

            #[derive(diesel::deserialize::QueryableByName, Debug)]
            struct FkInfo {
                #[diesel(sql_type = diesel::sql_types::Text)]
                name: String,
            }
            let fk_rows: Vec<FkInfo> = diesel::sql_query(
                "SELECT conname as name FROM pg_constraint c JOIN pg_class ON c.conrelid = pg_class.oid WHERE pg_class.relname = 'likes' AND contype = 'f' ORDER BY conname"
            )
            .load(&mut conn)
            .expect("Failed to query constraints");

            let fk_names: Vec<&str> =
                fk_rows.iter().map(|r| r.name.as_str()).collect();
            assert!(
                fk_names
                    .iter()
                    .any(|n| n.contains("likes_pet_owner_id_fkey")),
                "Missing FK, found: {:?}",
                fk_names
            );
            assert!(
                fk_names.iter().any(|n| n.contains("likes_pet_id_fkey")),
                "Missing FK, found: {:?}",
                fk_names
            );
            assert!(
                fk_names.iter().any(|n| n.contains("likes_user_id_fkey")),
                "Missing FK, found: {:?}",
                fk_names
            );
        }
    }

    #[actix_rt::test]
    async fn discover_next_returns_pet() {
        let ctx = TestContext::new(None).await;

        let token1 =
            register_and_login(&ctx, "discover_user1", "test123").await;
        let token2 =
            register_and_login(&ctx, "discover_user2", "test123").await;

        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token2)
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

        let mut resp = ctx
            .test_server
            .get("/api/v1/private/discover/next")
            .with_token(&token1)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["pet_uuid"], pet_uuid);
    }

    #[actix_rt::test]
    async fn discover_next_returns_empty_when_all_liked() {
        let ctx = TestContext::new(None).await;

        let token1 =
            register_and_login(&ctx, "discover_user3", "test123").await;
        let token2 =
            register_and_login(&ctx, "discover_user4", "test123").await;

        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token2)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Max",
                "species": "cat"
            }))
            .await
            .unwrap();
        let pet = create_resp.json::<serde_json::Value>().await.unwrap();
        let pet_uuid = pet["pet_uuid"].as_str().unwrap().to_string();

        let like_resp = ctx
            .test_server
            .post("/api/v1/private/likes")
            .with_token(&token1)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "pet_uuid": pet_uuid,
                "direction": "like"
            }))
            .await
            .unwrap();
        assert_eq!(like_resp.status(), StatusCode::CREATED);

        let resp = ctx
            .test_server
            .get("/api/v1/private/discover/next")
            .with_token(&token1)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[actix_rt::test]
    async fn discover_pets_returns_list() {
        let ctx = TestContext::new(None).await;

        let token1 =
            register_and_login(&ctx, "discover_user5", "test123").await;
        let token2 =
            register_and_login(&ctx, "discover_user6", "test123").await;

        for i in 0..3 {
            ctx.test_server
                .post("/api/v1/private/user/pets")
                .with_token(&token2)
                .append_header((CONTENT_TYPE, "application/json"))
                .send_json(&serde_json::json!({
                    "name": format!("Pet{}", i),
                    "species": "dog"
                }))
                .await
                .unwrap();
        }

        let mut resp = ctx
            .test_server
            .get("/api/v1/private/discover/pets")
            .with_token(&token1)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["pets"].as_array().is_some());
        let pets = body["pets"].as_array().unwrap();
        assert!(pets.len() >= 3);
    }

    #[actix_rt::test]
    async fn discover_pets_filters_by_species() {
        let ctx = TestContext::new(None).await;

        let token1 =
            register_and_login(&ctx, "discover_user7", "test123").await;
        let token2 =
            register_and_login(&ctx, "discover_user8", "test123").await;

        ctx.test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token2)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog"
            }))
            .await
            .unwrap();

        ctx.test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token2)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Whiskers",
                "species": "cat"
            }))
            .await
            .unwrap();

        let mut resp = ctx
            .test_server
            .get("/api/v1/private/discover/pets?species=dog")
            .with_token(&token1)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        let pets = body["pets"].as_array().unwrap();
        for pet in pets {
            assert_eq!(pet["species"], "dog");
        }
    }

    #[actix_rt::test]
    async fn create_like_success() {
        let ctx = TestContext::new(None).await;

        let token1 = register_and_login(&ctx, "like_user1", "test123").await;
        let token2 = register_and_login(&ctx, "like_user2", "test123").await;

        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token2)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog"
            }))
            .await
            .unwrap();
        let pet = create_resp.json::<serde_json::Value>().await.unwrap();

        let mut resp = ctx
            .test_server
            .post("/api/v1/private/likes")
            .with_token(&token1)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "pet_uuid": pet["pet_uuid"],
                "direction": "like"
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["direction"], "like");
        assert_eq!(body["is_match"], false);
    }

    #[actix_rt::test]
    async fn create_like_detects_match() {
        let ctx = TestContext::new(None).await;

        let token1 = register_and_login(&ctx, "match_user1", "test123").await;
        let token2 = register_and_login(&ctx, "match_user2", "test123").await;

        let mut create_resp1 = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token2)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog"
            }))
            .await
            .unwrap();
        let pet2 = create_resp1.json::<serde_json::Value>().await.unwrap();

        let mut create_resp2 = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token1)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Max",
                "species": "cat"
            }))
            .await
            .unwrap();
        let pet1 = create_resp2.json::<serde_json::Value>().await.unwrap();

        ctx.test_server
            .post("/api/v1/private/likes")
            .with_token(&token1)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "pet_uuid": pet2["pet_uuid"],
                "direction": "like"
            }))
            .await
            .unwrap();

        let mut resp = ctx
            .test_server
            .post("/api/v1/private/likes")
            .with_token(&token2)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "pet_uuid": pet1["pet_uuid"],
                "direction": "like"
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["is_match"], true);
    }

    #[actix_rt::test]
    async fn create_like_duplicate_returns_conflict() {
        let ctx = TestContext::new(None).await;

        let token1 = register_and_login(&ctx, "dup_user1", "test123").await;
        let token2 = register_and_login(&ctx, "dup_user2", "test123").await;

        let mut create_resp = ctx
            .test_server
            .post("/api/v1/private/user/pets")
            .with_token(&token2)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "name": "Buddy",
                "species": "dog"
            }))
            .await
            .unwrap();
        let pet = create_resp.json::<serde_json::Value>().await.unwrap();

        ctx.test_server
            .post("/api/v1/private/likes")
            .with_token(&token1)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "pet_uuid": pet["pet_uuid"],
                "direction": "like"
            }))
            .await
            .unwrap();

        let resp = ctx
            .test_server
            .post("/api/v1/private/likes")
            .with_token(&token1)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "pet_uuid": pet["pet_uuid"],
                "direction": "dislike"
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_rt::test]
    async fn stub_messages_returns_ok() {
        let ctx = TestContext::new(None).await;

        let token = register_and_login(&ctx, "msg_user1", "test123").await;

        let mut resp = ctx
            .test_server
            .post("/api/v1/private/messages")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "sender_id": 1,
                "receiver_id": 2,
                "content": "Hello"
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["status"], "stub");
    }

    #[actix_rt::test]
    async fn stub_reports_returns_ok() {
        let ctx = TestContext::new(None).await;

        let token = register_and_login(&ctx, "report_user1", "test123").await;

        let mut resp = ctx
            .test_server
            .post("/api/v1/private/reports")
            .with_token(&token)
            .append_header((CONTENT_TYPE, "application/json"))
            .send_json(&serde_json::json!({
                "reporter_id": 1,
                "target_id": 2,
                "reason": "spam"
            }))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["status"], "stub");
    }
}
