# Pet Image Gallery

## Overview

Add a pet image gallery system allowing users to upload, manage, and view multiple images per pet. Each upload generates three WebP variants (thumbnail 80×80, medium 400×400, original up to 2048×2048) stored in MinIO. One image per pet is marked as primary and exposed on the public pet view.

## Status: Planning

## Database Schema

### `pet_images` table

```sql
CREATE TABLE pet_images (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    pet_id INTEGER NOT NULL REFERENCES pets(id) ON DELETE CASCADE,
    thumbnail_key TEXT NOT NULL,
    medium_key TEXT NOT NULL,
    original_key TEXT NOT NULL,
    format VARCHAR(10) NOT NULL DEFAULT 'webp',
    is_primary BOOLEAN NOT NULL DEFAULT false,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pet_images_pet_id ON pet_images(pet_id);
```

**MinIO key pattern:** `pets/{uuid}/thumbnail.webp`, `pets/{uuid}/medium.webp`, `pets/{uuid}/original.webp`

## API Surface

### Pet image endpoints (authenticated owner)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/user/pets/{pet_id}/images` | RoleUser | Upload image (raw binary) |
| GET | `/api/user/pets/{pet_id}/images` | RoleUser | List pet's images |
| PATCH | `/api/user/pets/images/{uuid}/primary` | RoleUser | Set as primary |
| DELETE | `/api/user/pets/images/{uuid}` | RoleUser | Delete image (all variants) |

### Pet image endpoints (public)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/public/pets/images/{uuid}` | none | Get image metadata (no URLs) |
| GET | `/api/public/pets/images/{uuid}/{variant}` | none | Stream image variant (proxy) |

## Request/Response Models

**PublicPetImage:**
```rust
pub struct PublicPetImage {
    pub id: ImageId,
    pub uuid: Uuid,
    pub thumbnail_key: String,
    pub medium_key: String,
    pub original_key: String,
    pub format: String,
    pub is_primary: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}
```

**PetImageVariant:**
```rust
pub enum PetImageVariant {
    Thumbnail,
    Medium,
    Original,
}
```

**ListPetImagesResponse:**
```rust
pub struct ListPetImagesResponse {
    pub images: Vec<PublicPetImage>,
}
```

## Image Processing Pipeline

1. Validate MIME type (jpeg/png/webp only) using `infer`
2. Check file size against `max_pet_image_size_bytes` (default 5MB)
3. Decode image using `image` crate
4. Resize to fit within `pet_image_max_dimension` × `pet_image_max_dimension` (default 2048, maintain aspect ratio)
5. Encode three variants as WebP:
   - `original`: quality 85
   - `medium`: quality 80
   - `thumbnail`: quality 75
6. Upload all three to MinIO
7. Insert row into `pet_images` (auto-set `is_primary = true` if this is the pet's first image)

## Implementation Steps

### 1. Dependencies

Add to `Cargo.toml`:
```toml
image = { version = "0.25", features = ["webp"] }
```

### 2. Migration

- Create migration directory with `up.sql` (pet_images table + index) and `down.sql`
- Run migration, then `diesel print-schema` to generate schema.rs

### 3. Config

Add to `MinioConfig` in `config.rs`:
```rust
pub max_pet_image_size_bytes: u64,  // default 5MB
pub pet_image_max_dimension: u32,   // default 2048
```

### 4. Models (`models/pets.rs`)

- `ImageId` newtype (DieselNewType + Validator)
- `PetImage` — full queryable model
- `PublicPetImage` — response DTO
- `PetImageVariant` enum (Thumbnail, Medium, Original)
- Update `PublicPet` to include `primary_image: Option<PublicPetImage>`

### 5. Image Processing Utility (`utils/images.rs`)

- `resize_and_encode_webp(bytes, max_dimension) -> Result<ResizedImage>` — resizes and encodes all three variants
- Returns struct with three `Bytes` (thumbnail, medium, original) + format string

### 6. Actions (`actions/pets.rs`)

- `upload_pet_image(pet_id, user_id, bytes, conn, minio, config)` — ownership check → resize → encode → upload → insert DB
- `list_pet_images(pet_id, user_id, conn)` — ownership check → return images sorted by sort_order
- `delete_pet_image(image_uuid, user_id, conn)` — ownership check → delete 3 MinIO objects + DB row
- `set_primary_image(image_uuid, user_id, conn)` — ownership check → unset all primaries for pet → set this one

### 7. Routes (`routes/pets.rs`)

- Wire all 6 endpoints with utoipa path annotations
- Ownership enforcement: GET/PATCH/DELETE verify `user_id` matches pet owner
- Variant proxy endpoint: stream from MinIO with correct `Content-Type`
- `web::block` for all DB calls and image processing, `#[tracing::instrument]` on all handlers

### 8. Wiring

- Register routes in `src/lib.rs` under existing scopes
- Register OpenAPI schemas
- Update `get_public_pet` to include primary image

### 9. Tests

- Upload valid jpeg/png/webp → returns 201 with metadata
- Reject non-image MIME types (415)
- Reject oversized files (413)
- Verify resized dimensions fit within max dimension
- Verify all 3 variants uploaded to MinIO
- Ownership enforcement on all endpoints
- Delete removes all 3 MinIO objects
- First upload auto-sets primary
- Set primary toggles correctly (unsets previous, sets new)
- Proxy endpoint returns correct Content-Type and bytes

## Notes

- Images are stored in MinIO under `pets/{uuid}/` prefix
- WebP conversion is mandatory — all variants are .webp regardless of upload format
- Resize uses "fit within" (maintain aspect ratio), never crops
- First image uploaded to a pet is auto-set as primary
- On set-primary, all other images for the same pet are unset
- Proxy streaming follows the same pattern as `GET /api/public/avatars/{user_id}`
- No multipart — raw binary payload (same as user avatar upload)
