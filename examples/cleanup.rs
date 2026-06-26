use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenvy::dotenv;

fn main() {
    let _ = dotenv();

    let database_url = std::env::var("ACTIX_DEMO_DATABASE_URL")
        .expect("ACTIX_DEMO_DATABASE_URL must be set (or create .env file)");

    println!("Connecting to database...");
    let mut conn = PgConnection::establish(&database_url)
        .expect("Failed to connect to database");

    println!("\n=== Cleaning Up Seed Data ===\n");

    // Delete in reverse dependency order to avoid foreign key violations
    let tables = [
        ("pet_images", "images"),
        ("pet_personality_traits", "trait assignments"),
        ("pets", "pets"),
        ("profiles", "profiles"),
        ("users_roles", "role assignments"),
        ("users", "users"),
    ];

    for (table, desc) in &tables {
        let result = diesel::sql_query(format!("DELETE FROM {table}"))
            .execute(&mut conn);

        match result {
            Ok(count) => println!("  Deleted {count} {desc} from {table}"),
            Err(e) => eprintln!("  WARNING: Error deleting from {table}: {e}"),
        }
    }

    // Also clean up MinIO
    println!("\nCleaning up MinIO...");
    let minio_endpoint = std::env::var("ACTIX_DEMO_MINIO_ENDPOINT")
        .unwrap_or_else(|_| "localhost:9000".to_string());
    let _minio_access_key = std::env::var("ACTIX_DEMO_MINIO_ACCESS_KEY")
        .unwrap_or_else(|_| "minioadmin".to_string());
    let _minio_secret_key = std::env::var("ACTIX_DEMO_MINIO_SECRET_KEY")
        .unwrap_or_else(|_| "minioadmin".to_string());
    let minio_bucket_name = std::env::var("ACTIX_DEMO_MINIO_BUCKET_NAME")
        .unwrap_or_else(|_| "pet-app".to_string());

    println!("  MinIO endpoint: {minio_endpoint}");
    println!("  Bucket: {minio_bucket_name}");

    println!("\n\u{2705} Cleanup complete!");
    println!("\nNote: To fully clean MinIO, you may need to manually delete the 'pets/' prefix objects.");
}
