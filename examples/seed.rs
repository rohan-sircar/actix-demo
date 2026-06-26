use actix_demo::schema;
use actix_demo::models::roles::RoleEnum;
use actix_demo::models::pets::PetGender;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use uuid::Uuid as DieselUuid;
use bcrypt::{hash, DEFAULT_COST};
use minior::Minio;
use minior::aws_sdk_s3;
use minior::aws_sdk_s3::config::{Credentials, Region};
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use dotenvy::dotenv;

use schema::{users, users_roles, roles, profiles, pets, pet_personality_traits, personality_traits, pet_images};

fn get_role_user_id(conn: &mut PgConnection) -> QueryResult<i32> {
    roles::table
        .filter(schema::roles::role_name.eq(RoleEnum::RoleUser))
        .select(schema::roles::id)
        .first(conn)
}

fn seed_users_and_pets(conn: &mut PgConnection) -> QueryResult<(usize, usize, Vec<(&str, i32, DieselUuid)>)> {
    println!("\n=== Seeding Database ===\n");

    let role_user_id = get_role_user_id(conn).map_err(|e| {
        eprintln!("ERROR: Could not find role_user role: {e}");
        eprintln!("  Ensure migrations are run: diesel migration run");
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("role_user not found".to_string()),
        )
    })?;
    println!("  Found role_user (id={})", role_user_id);

    let seed_users = [
        (
            "swiper",
            "swiper@demo.com",
            "password123",
            "Swiper User",
            "I love discovering new pets! \u{1F43E}",
            "San Francisco, CA",
        ),
        (
            "petowner1",
            "owner1@demo.com",
            "password123",
            "Alice Johnson",
            "Dog mom of 4 wonderful pups \u{1F415}",
            "Oakland, CA",
        ),
        (
            "petowner2",
            "owner2@demo.com",
            "password123",
            "Bob Smith",
            "Cat dad and bird enthusiast \u{1F426}",
            "Berkeley, CA",
        ),
    ];

    let mut user_ids: Vec<(&str, i32)> = Vec::new();

    println!("Creating users...");
    for (username, email, password, display_name, bio, location) in &seed_users {
        let existing = users::table
            .filter(schema::users::username.eq(username))
            .select(schema::users::id)
            .first::<i32>(conn)
            .optional()?;

        if let Some(id) = existing {
            println!("  User '{username}' already exists (id={id})");
            user_ids.push((username, id));
        } else {
            let password_hash = hash(password, DEFAULT_COST).expect("Failed to hash password");
            let user_uuid = DieselUuid::new_v4();

            diesel::insert_into(users::table)
                .values((
                    schema::users::username.eq(username),
                    schema::users::password.eq(&password_hash),
                    schema::users::email.eq(email),
                    schema::users::user_uuid.eq(user_uuid),
                    schema::users::email_verified.eq(true),
                ))
                .execute(conn)?;

            let id = users::table
                .filter(schema::users::username.eq(username))
                .select(schema::users::id)
                .first(conn)?;

            diesel::insert_into(users_roles::table)
                .values((
                    schema::users_roles::user_id.eq(id),
                    schema::users_roles::role_id.eq(role_user_id),
                ))
                .execute(conn)?;

            diesel::insert_into(profiles::table)
                .values((
                    schema::profiles::user_id.eq(id),
                    schema::profiles::display_name.eq(display_name),
                    schema::profiles::bio.eq(Some(*bio)),
                    schema::profiles::location.eq(Some(*location)),
                ))
                .execute(conn)?;

            println!("  Created user '{username}' (id={id})");
            user_ids.push((username, id));
        }
    }

    let swiper_id = user_ids.iter().find(|(u, _)| *u == "swiper").unwrap().1;
    let owner1_id = user_ids.iter().find(|(u, _)| *u == "petowner1").unwrap().1;
    let owner2_id = user_ids.iter().find(|(u, _)| *u == "petowner2").unwrap().1;

    println!("\nUser IDs: swiper={swiper_id}, owner1={owner1_id}, owner2={owner2_id}");

    let trait_names = [
        "playful", "calm", "energetic", "affectionate", "independent",
        "curious", "loyal", "gentle", "social", "smart",
    ];

    let trait_ids: Vec<i32> = personality_traits::table
        .filter(schema::personality_traits::name.eq_any(&trait_names))
        .select(schema::personality_traits::id)
        .load(conn)?;

    println!("  Found {} personality traits", trait_ids.len());

    let owner1_pets = [
        ("Buddy", "Dog", "Golden Retriever", PetGender::Male, Some(65.5), "Golden", "Buddy is the friendliest dog you'll ever meet!"),
        ("Luna", "Dog", "French Bulldog", PetGender::Female, Some(22.0), "Brindle", "Luna loves naps and cuddles"),
        ("Max", "Dog", "Corgi", PetGender::Male, Some(28.0), "Orange and White", "Max is a herding legend in a small package"),
        ("Bella", "Dog", "Labrador", PetGender::Female, Some(60.0), "Black", "Bella is a water lover and obedience champion"),
        ("Charlie", "Dog", "Border Collie", PetGender::Male, Some(50.0), "Black and White", "Charlie is the Einstein of dogs"),
        ("Daisy", "Cat", "Siamese", PetGender::Female, Some(8.5), "Cream and Brown", "Daisy is vocal and loves attention"),
    ];

    let owner2_pets = [
        ("Whiskers", "Cat", "Maine Coon", PetGender::Male, Some(18.0), "Gray Tabby", "Whiskers is a gentle giant"),
        ("Mittens", "Cat", "Orange Tabby", PetGender::Female, Some(10.0), "Orange", "Mittens is playful and loves toys"),
        ("Cleo", "Cat", "Persian", PetGender::Female, Some(9.0), "White", "Cleo is a diva who loves luxury"),
        ("Tweety", "Bird", "Cockatiel", PetGender::Male, Some(0.08), "Yellow", "Tweety loves to whistle and sing"),
        ("Polly", "Bird", "African Grey", PetGender::Female, Some(0.12), "Gray", "Polly can say 50+ words"),
        ("Rocky", "Cat", "British Shorthair", PetGender::Male, Some(14.0), "Blue", "Rocky is chill and independent"),
    ];

    let mut all_pet_ids: Vec<i32> = Vec::new();
    let mut pet_uuids: Vec<(&str, i32, DieselUuid)> = Vec::new();

    println!("\nCreating pets for petowner1...");
    for (name, species, breed, gender, weight, color, description) in &owner1_pets {
        let existing = pets::table
            .filter(schema::pets::name.eq(name))
            .filter(schema::pets::user_id.eq(owner1_id))
            .select((schema::pets::id, schema::pets::pet_uuid))
            .first::<(i32, DieselUuid)>(conn)
            .optional()?;

        if let Some((id, pet_uuid)) = existing {
            println!("  Pet '{name}' already exists (id={id})");
            all_pet_ids.push(id);
            pet_uuids.push((name, id, pet_uuid));
        } else {
            let pet_uuid = DieselUuid::new_v4();
            diesel::insert_into(pets::table)
                .values((
                    schema::pets::pet_uuid.eq(pet_uuid),
                    schema::pets::user_id.eq(owner1_id),
                    schema::pets::name.eq(name),
                    schema::pets::species.eq(species),
                    schema::pets::breed.eq(Some(*breed)),
                    schema::pets::gender.eq(Some(gender.clone())),
                    schema::pets::weight.eq(*weight),
                    schema::pets::color_markings.eq(Some(*color)),
                    schema::pets::description.eq(Some(*description)),
                ))
                .execute(conn)?;

            let id = pets::table
                .filter(schema::pets::name.eq(name))
                .filter(schema::pets::user_id.eq(owner1_id))
                .select(schema::pets::id)
                .first(conn)?;

            println!("  Created pet '{name}' (id={id})");
            all_pet_ids.push(id);
            pet_uuids.push((name, id, pet_uuid));
        }
    }

    println!("\nCreating pets for petowner2...");
    for (name, species, breed, gender, weight, color, description) in &owner2_pets {
        let existing = pets::table
            .filter(schema::pets::name.eq(name))
            .filter(schema::pets::user_id.eq(owner2_id))
            .select((schema::pets::id, schema::pets::pet_uuid))
            .first::<(i32, DieselUuid)>(conn)
            .optional()?;

        if let Some((id, pet_uuid)) = existing {
            println!("  Pet '{name}' already exists (id={id})");
            all_pet_ids.push(id);
            pet_uuids.push((name, id, pet_uuid));
        } else {
            let pet_uuid = DieselUuid::new_v4();
            diesel::insert_into(pets::table)
                .values((
                    schema::pets::pet_uuid.eq(pet_uuid),
                    schema::pets::user_id.eq(owner2_id),
                    schema::pets::name.eq(name),
                    schema::pets::species.eq(species),
                    schema::pets::breed.eq(Some(*breed)),
                    schema::pets::gender.eq(Some(gender.clone())),
                    schema::pets::weight.eq(*weight),
                    schema::pets::color_markings.eq(Some(*color)),
                    schema::pets::description.eq(Some(*description)),
                ))
                .execute(conn)?;

            let id = pets::table
                .filter(schema::pets::name.eq(name))
                .filter(schema::pets::user_id.eq(owner2_id))
                .select(schema::pets::id)
                .first(conn)?;

            println!("  Created pet '{name}' (id={id})");
            all_pet_ids.push(id);
            pet_uuids.push((name, id, pet_uuid));
        }
    }

    println!("\nAssigning personality traits...");
    for (i, pet_id) in all_pet_ids.iter().enumerate() {
        let num_traits = 2 + (i % 3);
        let start_idx = (i * 2) % trait_ids.len();
        let trait_subset: Vec<i32> = trait_ids
            .iter()
            .cycle()
            .skip(start_idx)
            .take(num_traits)
            .cloned()
            .collect();

        for trait_id in &trait_subset {
            let count: i64 = pet_personality_traits::table
                .filter(schema::pet_personality_traits::pet_id.eq(*pet_id))
                .filter(schema::pet_personality_traits::trait_id.eq(*trait_id))
                .count()
                .first(conn)
                .unwrap_or(0);

            if count == 0 {
                diesel::insert_into(pet_personality_traits::table)
                    .values((
                        schema::pet_personality_traits::pet_id.eq(*pet_id),
                        schema::pet_personality_traits::trait_id.eq(*trait_id),
                    ))
                    .execute(conn)?;
            }
        }
    }

    Ok((user_ids.len(), pet_uuids.len(), pet_uuids))
}

async fn seed_images(minio: &Minio, bucket_name: &str, seed_images_dir: &PathBuf, pet_uuids: &[(String, i32, DieselUuid)], conn: &mut PgConnection) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Seeding Pet Images ===\n");

    for (pet_name, pet_id, pet_uuid) in pet_uuids.iter() {
        let pet_dir = seed_images_dir.join(pet_name.to_lowercase());
        if !pet_dir.exists() {
            println!("  Skipped '{pet_name}' - no image directory found");
            continue;
        }

        let mut image_records: Vec<(DieselUuid, String, String, String)> = Vec::new();

        // Read all image files in the pet directory
        let mut entries: Vec<_> = fs::read_dir(&pet_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ["jpg", "jpeg", "png", "webp"].contains(&ext.to_string_lossy().to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in &entries {
            let image_uuid = DieselUuid::new_v4();
            let file_path = entry.path();
            let file_bytes = fs::read(&file_path)?;

            let thumbnail_key = format!("pets/{pet_uuid}/{image_uuid}/thumbnail.webp");
            let medium_key = format!("pets/{pet_uuid}/{image_uuid}/medium.webp");
            let original_key = format!("pets/{pet_uuid}/{image_uuid}/original.webp");

            // Upload to all three variants in MinIO (same file, different keys)
            minio
                .client
                .put_object()
                .bucket(bucket_name)
                .key(&thumbnail_key)
                .body(bytes::Bytes::from(file_bytes.clone()).into())
                .send()
                .await?;

            minio
                .client
                .put_object()
                .bucket(bucket_name)
                .key(&medium_key)
                .body(bytes::Bytes::from(file_bytes.clone()).into())
                .send()
                .await?;

            minio
                .client
                .put_object()
                .bucket(bucket_name)
                .key(&original_key)
                .body(bytes::Bytes::from(file_bytes).into())
                .send()
                .await?;

            println!("  Uploaded '{pet_name}' image {} → {thumbnail_key}, {medium_key}, {original_key}", image_records.len() + 1);
            image_records.push((image_uuid, thumbnail_key, medium_key, original_key));
        }

        // Insert pet_images records
        for (i, (image_uuid, thumbnail_key, medium_key, original_key)) in image_records.iter().enumerate() {
            diesel::insert_into(pet_images::table)
                .values((
                    schema::pet_images::uuid.eq(*image_uuid),
                    schema::pet_images::pet_id.eq(*pet_id),
                    schema::pet_images::thumbnail_key.eq(thumbnail_key),
                    schema::pet_images::medium_key.eq(medium_key),
                    schema::pet_images::original_key.eq(original_key),
                    schema::pet_images::format.eq("webp"),
                    schema::pet_images::is_primary.eq(i == 0),
                    schema::pet_images::sort_order.eq(i as i32),
                ))
                .execute(conn)?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = dotenv();

    let database_url = std::env::var("ACTIX_DEMO_DATABASE_URL")
        .expect("ACTIX_DEMO_DATABASE_URL must be set (or create .env file)");

    let minio_endpoint = std::env::var("ACTIX_DEMO_MINIO_ENDPOINT")
        .unwrap_or_else(|_| "localhost:9000".to_string());
    let minio_access_key = std::env::var("ACTIX_DEMO_MINIO_ACCESS_KEY")
        .unwrap_or_else(|_| "minioadmin".to_string());
    let minio_secret_key = std::env::var("ACTIX_DEMO_MINIO_SECRET_KEY")
        .unwrap_or_else(|_| "minioadmin".to_string());
    let minio_bucket_name = std::env::var("ACTIX_DEMO_MINIO_BUCKET_NAME")
        .unwrap_or_else(|_| "pet-app".to_string());

    println!("Connecting to database...");
    let mut conn1 = PgConnection::establish(&database_url)
        .expect("Failed to connect to database");

    let (user_count, pet_count, pet_uuids_raw) = seed_users_and_pets(&mut conn1)
        .expect("Failed to seed users and pets");
    
    // Clone to owned strings to break the borrow
    let pet_uuids: Vec<(String, i32, DieselUuid)> = pet_uuids_raw
        .into_iter()
        .map(|(name, id, uuid)| (name.to_string(), id, uuid))
        .collect();
    
    drop(conn1); // Close connection before opening new one

    // Setup MinIO client
    println!("\nConnecting to MinIO...");
    let cred = Credentials::new(
        &minio_access_key,
        &minio_secret_key,
        None,
        None,
        "seed-script",
    );

    let endpoint_url = if minio_endpoint.contains("://") {
        minio_endpoint.clone()
    } else {
        format!("http://{minio_endpoint}")
    };

    let s3_config = aws_sdk_s3::config::Config::builder()
        .endpoint_url(&endpoint_url)
        .credentials_provider(cred)
        .region(Region::new("custom-local"))
        .force_path_style(true)
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .build();

    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

    let minio = Minio {
        client: Arc::new(s3_client),
    };

    // Check if MinIO is reachable
    let minio_reachable = minio.client.list_buckets().send().await.is_ok();
    if minio_reachable {
        println!("  Connected to MinIO at {minio_endpoint}");
    } else {
        eprintln!("  WARNING: Could not connect to MinIO. Skipping image seeding.");
    }

    // Find seed_images directory relative to the example file
    let seed_images_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("seed_images");

    if !seed_images_dir.exists() {
        eprintln!("  WARNING: Seed images directory not found at {seed_images_dir:?}");
        eprintln!("  Skipping image seeding.");
    } else if minio_reachable {
        let mut conn2 = PgConnection::establish(&database_url)
            .expect("Failed to connect to database for image seeding");
        match seed_images(&minio, &minio_bucket_name, &seed_images_dir, &pet_uuids, &mut conn2).await {
            Ok(()) => println!("\n\u{2705} Images seeded successfully!"),
            Err(e) => eprintln!("\n\u{274C} Error seeding images: {e}"),
        }
    }

    println!("\n=== Seeding Complete ===");
    println!("  Users: {user_count}");
    println!("  Pets: {pet_count}");
    println!("\nYou can now login as:");
    println!("  Username: swiper, Password: password123");
    println!("  Username: petowner1, Password: password123");
    println!("  Username: petowner2, Password: password123");
}
