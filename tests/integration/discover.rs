mod discover {
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

        // Create two test users in the DB directly
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

        // Create a test pet for user2 so we can like it
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

        // Insert a like record using the NewLike model — verifies the likes table schema
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

        // Query back the like record using the Like model — verifies all columns are correct
        let liked_likes: Vec<Like> = likes_schema::table
            .order(likes_schema::id.asc())
            .load(&mut conn)
            .expect("Failed to query likes table");

        assert_eq!(liked_likes.len(), 1);
        let like = &liked_likes[0];
        assert_eq!(like.direction, LikeDirection::Like);
        assert_eq!(like.is_match, false); // no reciprocal like yet

        // Verify indexes exist via pg_indexes using diesel's QueryableByName
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

        // Verify FK constraints exist via pg_constraint
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
