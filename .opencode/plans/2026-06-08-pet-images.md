# Pet Image Gallery (Phase 2)

## Overview

Add a pet image gallery system allowing users to upload, manage, and view multiple images per pet. Each upload generates three WebP variants (thumbnail 80x80, medium 400x400, original up to 2048x2048) stored in MinIO. One image per pet is marked as primary and exposed on the public pet view.

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

**MinIO key pattern:** `pets/{pet_uuid}/thumbnail.webp`, `pets/{pet_uuid}/medium.webp`, `pets/{pet_uuid}/original.webp`

## API Surface

### Pet image endpoints (authenticated owner)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/user/pets/{pet_uuid}/images` | RoleUser | Upload image (raw binary) |
| GET | `/api/user/pets/{pet_uuid}/images` | RoleUser | List pet's images |
| DELETE | `/api/user/pets/{pet_uuid}/images/{image_uuid}` | RoleUser | Delete image (all variants) |
| PATCH | `/api/user/pets/{pet_uuid}/images/{image_uuid}` | RoleUser | Update image (set primary via body `{"is_primary": true}`) |

### Pet image endpoints (public)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/public/pets/images/{image_uuid}` | none | Get image metadata (no keys) |
| GET | `/api/public/pets/images/{image_uuid}/{variant}` | none | Stream image variant (proxy) |

## Request/Response Models

**PublicPetImage:**
```rust
pub struct PublicPetImage {
    pub id: ImageId,
    pub uuid: Uuid,
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

**UploadPetImageRequest:** (for utoipa schema only)
```rust
pub struct UploadPetImageRequest {
    pub file: Vec<u8>,
}
```

## Image Processing Pipeline

1. Validate MIME type (jpeg/png/webp only) using `infer` — reuse existing `validate_image_stream` utility with pet-specific max size
2. Check file size against `max_pet_image_size_bytes` (default 5MB)
3. Decode image using `image` crate
4. Resize to fit within `pet_image_max_dimension` x `pet_image_max_dimension` (default 2048, maintain aspect ratio)
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

Create `migrations/2026-06-08-000000_create_pet_images/up.sql` and `down.sql`.
Run migration, then `diesel print-schema` to generate schema.rs.

### 3. Config

Add to `MinioConfig` in `config.rs`:
```rust
pub max_pet_image_size_bytes: u64,  // default 5MB
```

### 4. Models (`src/models/pets.rs`)

- `ImageId` newtype (DieselNewType + Validator, positive int values like PetId/TraitId)
- `PetImage` — full queryable model with all columns
- `PublicPetImage` — response DTO (no keys in public response)
- `PetImageVariant` enum (Thumbnail, Medium, Original)
- `UploadPetImageRequest` — utoipa schema for upload endpoint
- Update `PublicPet` to include `primary_image: Option<PublicPetImage>`

### 5. Image Processing Utility (`src/utils/images.rs`) — new file

- `resize_and_encode_webp(bytes, max_dimension) -> Result<ResizedImage>` — resizes and encodes all three variants
- Returns struct with three `Bytes` (thumbnail, medium, original) + format string

### 6. Actions (`src/actions/pets.rs`)

- `upload_pet_image(pet_uuid, user_id, bytes, conn, minio, config)` — lookup pet by UUID → ownership check → resize → encode → upload → insert DB (auto-set primary if first)
- `list_pet_images(pet_uuid, user_id, conn)` — lookup pet by UUID → ownership check → return images sorted by sort_order
- `delete_pet_image(pet_uuid, image_uuid, user_id, conn)` — lookup pet by UUID → verify ownership → find image → delete 3 MinIO objects + DB row → renumber sort_order
- `set_primary_image(pet_uuid, image_uuid, user_id, conn)` — lookup pet by UUID → verify ownership → find image → unset all primaries for pet → set this one
- Update `get_public_pet` to fetch primary image
- Update `fetch_all_traits_for_pets` batch query to also fetch primary images (single JOIN to avoid N+1)

### 7. Routes (`src/routes/pets.rs`)

- Wire all 6 endpoints with utoipa path annotations
- DELETE `/api/user/pets/{pet_uuid}/images/{image_uuid}` — delete a specific image within pet's collection
- PATCH `/api/user/pets/{pet_uuid}/images/{image_uuid}` — update image (body `{"is_primary": true}` to set primary)
- Ownership enforcement: all authenticated endpoints verify `user_id` matches pet owner via `pet_uuid`
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
- `DELETE /api/user/pets/{pet_uuid}/images/{image_uuid}` removes all 3 MinIO objects and renumbers sort_order
- First upload auto-sets primary
- `PATCH /api/user/pets/{pet_uuid}/images/{image_uuid}` with `{"is_primary": true}` toggles correctly (unsets previous, sets new)
- Proxy endpoint returns correct Content-Type and bytes
- Primary image included in PublicPet responses

## Key Design Decisions

### UUID-first routing

- Authenticated endpoints use `pet_uuid` for route params (not `pet_id`) — consistent with pet profiles Phase 1
- Public endpoints use `image_uuid` for route params (not `pet_uuid` or `image_id`) — avoids enumeration attacks
- `pet_images.pet_id` column uses INTEGER FK to `pets.id` — consistent with junction table pattern

### Primary image strategy

- Primary image metadata included in all `PublicPet` responses (list/get) — avoids N+1 when displaying pet cards with avatars
- Single batched JOIN query fetches primary images alongside traits (same pattern as trait fetching)

### Sort order management

- On delete, renumber remaining images to avoid gaps — simpler for future sorting
- On upload, auto-increment from max existing sort_order + 1

### Image validation

- Reuse existing `validate_image_stream` utility from avatar system
- Different max size config: avatars 2MB, pet images 5MB
- Same allowed MIME types: jpeg/png/webp

### MinIO key pattern

- `pets/{pet_uuid}/{variant}.webp` — uses pet UUID (not pet_id) for object keys
- Consistent with UUID-first design philosophy across the application

## Notes

- Images are stored in MinIO under `pets/{pet_uuid}/` prefix
- WebP conversion is mandatory — all variants are .webp regardless of upload format
- Resize uses "fit within" (maintain aspect ratio), never crops
- First image uploaded to a pet is auto-set as primary
- On set-primary, all other images for the same pet are unset
- Proxy streaming follows the same pattern as `GET /api/public/avatars/{user_id}`
- No multipart — raw binary payload (same as user avatar upload)
- Image processing wrapped in `web::block` (CPU-bound operation)
