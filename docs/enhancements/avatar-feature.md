# User Avatar Feature Implementation

## Objective

Implement secure avatar upload/retrieval using MinIO without database schema changes by leveraging deterministic object paths.

## Implementation Strategy

### Object Storage Configuration

The system uses MinIO for object storage. The following environment variables need to be configured in `.env`:

```bash
ACTIX_DEMO_MINIO_ENDPOINT=http://localhost:9000
ACTIX_DEMO_MINIO_ACCESS_KEY=minio
ACTIX_DEMO_MINIO_SECRET_KEY=minio
ACTIX_DEMO_MINIO_SECURE=false
ACTIX_DEMO_MINIO_BUCKET=actix_demo
ACTIX_DEMO_MAX_AVATAR_SIZE=2097152  # 2MB in bytes
```

Add to `src/config.rs`:

```rust
pub struct ObjectStorageConfig {
    // MinIO endpoint URL
    pub minio_endpoint: String,

    // MinIO access credentials
    pub minio_access_key: String,
    pub minio_secret_key: String,

    // Use HTTPS for MinIO
    pub minio_secure: bool,

    // Bucket name for avatars
    pub bucket_name: String,

    // Maximum avatar size in bytes
    #[serde(default = "default_avatar_size_limit")]
    pub max_avatar_size_bytes: u64,
}

fn default_avatar_size_limit() -> u64 {
    2 * 1024 * 1024 // 2MB
}
```

### Object Storage Path Format

```rust
// Path format: avatars/{user_id}.{ext}
let object_key = format!("avatars/{}.{}", user_id.as_uint(), file_extension);
```

### API Endpoints

1. **Upload Avatar** (`PUT /users/me/avatar`)

   - Content-Type: `multipart/form-data`
   - Auth-required
   - Validation:
     - Max size: 2MB
     - MIME type verification using `infer` crate to check magic numbers
     - Allowed types: image/jpeg, image/png, image/webp

2. **Get Avatar** (`GET /users/{id}/avatar`)
   - Retrieve the user's avatar from the MinIO bucket as a stream
   - Cache-Control: public, max-age=604800
   - Content-Disposition: `inline; filename="{user_id}.{ext}"`

### Security Measures

- Server-side MIME type validation via `infer` crate
- Size validation before processing
- Rate limiting applied via existing middleware (configurable in `src/config.rs`)

### Error Handling

Update `src/errors.rs` to include:

```rust
pub enum DomainError {
    // ... existing variants ...
    ObjectStorageConfigError,
    InvalidMimeType { detected: String },
    AvatarUploadFailed(anyhow::Error),
    FileSizeExceeded { max_bytes: u64 },
}
```

### Dependencies

Add to Cargo.toml:

```toml
[dependencies]
minio = "0.10.0"
tokio-util = { version = "0.7", features = ["io"] }
infer = "0.13.0"
```

### Rate Limiting Configuration

Update `src/config.rs` to include:

```rust
#[serde(default = "default_rate_limit_avatar_max_requests")]
pub rate_limit_avatar_max_requests: u32,

#[serde(default = "default_rate_limit_avatar_window_secs")]
pub rate_limit_avatar_window_secs: u64,
```

Add defaults to `src/models/defaults.rs`:

```rust
pub fn default_rate_limit_avatar_max_requests() -> u32 {
    5
}

pub fn default_rate_limit_avatar_window_secs() -> u64 {
    60
}
```

also add env vars to `src/config.rs`

### Implementation Plan

1. [x] Add cargo dependencies (minior)
2. [x] Add minio docker service to docker-compose.yaml file
3. [x] Add minio config values to AppConfig and EnvConfig and .env
4. [x] Add minio client to AppData in lib.rs
   1. [x] Create minio client in main.rs using env config values and add the instance to app_data
5. [x] Add Error types to errors.rs
6. [x] Create avatar upload endpoint in routes/users.rs
   1. [x] Add magic number check for image types
7. [x] Create avatar get endpoint in routes/users.rs

### Testing Plan

1. Integration tests for:
   - MIME type validation (valid/invalid cases)
   - Rate limiting enforcement
   - Presigned URL generation and expiration
2. End-to-end test for full upload/retrieval flow
3. Load testing for concurrent uploads
