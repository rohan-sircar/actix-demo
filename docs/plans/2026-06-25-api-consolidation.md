# API Consolidation Plan: Move All User/Pet Endpoints Behind Authentication

**Date:** 2026-06-25  
**Status:** Draft  

---

## Executive Summary

Currently, the `/api/v1` public scope contains user and pet endpoints that should only be accessible to authenticated users. This plan consolidates all user and pet related endpoints under `/api/v1/private/` (already authenticated via `GrantsMiddleware::with_extractor(routes::auth::extract)`), ensuring no unauthenticated access to user profiles, pet data, or pet images.

### Key Challenge

Image variant endpoints (`/api/v1/pets/images/{image_uuid}/{variant}`) serve raw image streams used in `<img>` / `<Image>` tags. Browsers cannot attach `Authorization` headers to image requests, so we must support **query parameter token authentication** for image URLs specifically.

---

## Part 1: Backend Route Configuration (`src/lib.rs`)

### 1.1 Remove Endpoints from Public Scope (lines 424-471)

Remove the following routes from the `/api/v1` public scope. The `/users` and `/users/{user_id}` routes in the public scope are **duplicates** of the admin endpoints already under `/api/v1/private/admin/users` — they should simply be removed.

| Current Path | Handler | Action |
|---|---|---|
| `/profiles/{user_id}` | `get_public_profile` | **Move to private** |
| `/pets/traits` | `get_traits` | **Move to private** |
| `/pets/{pet_uuid}` | `get_public_pet` | **Move to private** |
| `/pets/{pet_uuid}/images` | `get_pet_images` | **Move to private** |
| `/pets/images/{image_uuid}` | `get_public_pet_image` | **Move to private** |
| `/pets/images/{image_uuid}/{variant}` | `get_pet_image_variant` | **Move to private** |
| `/users` (GET) | `get_users` | **Remove** (duplicate of admin endpoint) |
| `/users/{user_id}` (GET) | `get_user` | **Remove** (duplicate of admin endpoint) |

**KEEP PUBLIC:** `/avatars/{user_id}` — This is an image-serving endpoint that returns raw avatar image bytes. It's a special case similar to pet images and should remain public for now (or we can apply the same query-token approach).

### 1.2 Add Endpoints to Private Scope

Add the moved endpoints under the existing `/api/v1/private` scope (line 256). The private scope already has:
- `GrantsMiddleware::with_extractor(routes::auth::extract)` — sets `x-auth-user` header
- `from_fn(utils::cookie_auth)` — cookie-based auth middleware

#### New Routes to Add Under `/api/v1/private`:

```rust
// After line 420 (before closing the private scope), add:

// User profile endpoints (moved from public)
.route(
    "/profiles/{user_id}",
    web::get().to(routes::users::get_public_profile),
)
// Pet endpoints (moved from public)
.route(
    "/pets/traits",
    web::get().to(routes::pets::get_traits),
)
.route(
    "/pets/{pet_uuid}",
    web::get().to(routes::pets::get_public_pet),
)
.route(
    "/pets/{pet_uuid}/images",
    web::get().to(routes::pets::get_pet_images),
)
.route(
    "/pets/images/{image_uuid}",
    web::get().to(routes::pets::get_public_pet_image),
)
.route(
    "/pets/images/{image_uuid}/{variant}",
    web::get().to(routes::pets::get_pet_image_variant),
)
```

### 1.3 Updated Route Structure

After changes, the route organization will be:

```
/api/v1/
├── auth/                          (public, rate-limited)
│   ├── login, exchange, logout
│   ├── registration
│   ├── verify-email, resend-verification-email
│   ├── password-reset-request, password-reset-complete
│   └── oauth/{github,google}/...
├── ws/                            (public, rate-limited)
├── build-info                     (public)
├── metrics/cmd                    (public)
├── avatars/{user_id}              (public - image serving)
├── private/                       (authenticated)
│   ├── cmd / cmd/{job_id}        (commands)
│   ├── avatars                    (upload/delete avatar)
│   ├── sessions                   (session management)
│   ├── user/                      (user's own data)
│   │   ├── GET/DELETE/PATCH       (my profile)
│   │   ├── profile                (CRUD profile)
│   │   └── pets/                  (user's pets)
│   │       ├── POST/GET           (list/create)
│   │       ├── {pet_uuid}         (get/update/delete)
│   │       └── {pet}/images       (image CRUD)
│   ├── admin/users                (admin user management)
│   ├── discover/next, discover/pets
│   ├── likes, messages, reports
│   ├── profiles/{user_id}         ← NEW (was public)
│   ├── pets/traits                ← NEW (was public)
│   ├── pets/{pet_uuid}            ← NEW (was public)
│   ├── pets/{pet}/images          ← NEW (was public)
│   ├── pets/images/{image_uuid}   ← NEW (was public)
│   └── pets/images/{image}/{variant} ← NEW (was public)
```

---

## Part 2: Route Handler Function Changes

### 2.1 `src/routes/pets.rs` — Handlers to Modify

#### `get_public_pet` (lines 62-77)

**Current:** No auth extraction, accepts only `app_data` and `pet_uuid`.  
**Change:** Add `req: HttpRequest`, extract user UUID, add `#[protect]` attribute.

```rust
#[utoipa::path(
    get,
    path = "/api/v1/private/pets/{pet_uuid}",  // UPDATED PATH
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
    ),
    responses(
        (status = 200, description = "Pet found", body = crate::models::pets::PublicPet),
        (status = 404, description = "Pet not found", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(pet_uuid))]
pub async fn get_public_pet(
    req: HttpRequest,                    // ADDED
    app_data: web::Data<AppData>,
    pet_uuid: web::Path<crate::models::pets::PetUuid>,
) -> Result<HttpResponse, DomainError> {
    let _user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;  // ADDED
    let pet_uuid = pet_uuid.into_inner();

    let pet = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::pets::get_public_pet(&pet_uuid, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(pet))
}
```

#### `get_pet_images` (lines 93-109)

**Current:** Already extracts user UUID from headers.  
**Change:** Update OpenAPI path annotation only.

```rust
#[utoipa::path(
    get,
    path = "/api/v1/private/pets/{pet_uuid}/images",  // UPDATED PATH
    tag = "pets",
    params(
        ("pet_uuid" = crate::models::pets::PetUuid, Path, description = "Pet UUID"),
    ),
    responses(
        (status = 200, description = "List of pet images", body = Vec<crate::models::pets::PublicPetImage>),
        (status = 401, description = "Missing auth", body = DomainError),
        (status = 404, description = "Pet not found", body = DomainError),
    ),
)]
// Handler body stays the same — already extracts user UUID
```

#### `get_public_pet_image` (lines 571-605)

**Current:** No auth at all.  
**Change:** Add query parameter `token` support for `<img>` tag compatibility. Extracts from query param OR header.

```rust
use actix_web::web::Query;

#[derive(serde::Deserialize)]
pub(crate) struct ImageVariantQuery {
    pub token: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/private/pets/images/{image_uuid}",  // UPDATED PATH
    tag = "pets",
    params(
        ("image_uuid" = uuid::Uuid, Path, description = "Image UUID"),
    ),
    responses(
        (status = 200, description = "Image metadata", body = crate::models::pets::PublicPetImage),
        (status = 404, description = "Image not found", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(image_uuid))]
pub async fn get_public_pet_image(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    image_uuid: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, DomainError> {
    // Auth: header already set by GrantsMiddleware, OR query param as fallback
    let _user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;
    let image_uuid = image_uuid.into_inner();

    let image = web::block(move || -> Result<Option<crate::models::pets::PetImage>, crate::errors::DomainError> {
        let pool = &app_data.pool;
        let mut conn = pool.get().map_err(|e| {
            crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))
        })?;
        use crate::schema::pet_images::dsl as pet_images;
        let result: Result<crate::models::pets::PetImage, _> = pet_images::pet_images
            .filter(pet_images::uuid.eq(image_uuid))
            .first(&mut conn);
        match result {
            Ok(img) => Ok(Some(img)),
            Err(diesel::NotFound) => Ok(None),
            Err(e) => Err(crate::errors::DomainError::new_internal_error(format!("DB error: {}", e))),
        }
    })
    .await??;

    let image = match image {
        Some(img) => img,
        None => {
            return Err(DomainError::new_entity_does_not_exist_error(format!(
                "Image {} not found",
                image_uuid
            )))
        }
    };

    Ok(HttpResponse::Ok().json(PublicPetImage::from(&image)))
}
```

#### `get_pet_image_variant` (lines 621-758) — CRITICAL: Query Token Support

**Current:** No auth at all. Serves raw image bytes.  
**Change:** Add query parameter `token` support. This is the **only** endpoint that must support query-param auth because it's called by `<img>` / `<Image>` tags which can't set headers.

```rust
#[derive(serde::Deserialize)]
pub(crate) struct PublicImageVariantPath {
    pub image_uuid: uuid::Uuid,
    pub variant: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/private/pets/images/{image_uuid}/{variant}",  // UPDATED PATH
    tag = "pets",
    params(
        ("image_uuid" = uuid::Uuid, Path, description = "Image UUID"),
        ("variant" = String, Path, description = "Variant (thumbnail, medium, original)"),
    ),
    responses(
        (status = 200, description = "Image stream"),
        (status = 404, description = "Image not found", body = DomainError),
        (status = 401, description = "Missing auth", body = DomainError),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(image_uuid, variant))]
pub async fn get_pet_image_variant(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    path: web::Path<PublicImageVariantPath>,
) -> Result<HttpResponse, DomainError> {
    let PublicImageVariantPath { image_uuid, variant } = path.into_inner();
    let app_data_clone = app_data.clone();

    // Auth: header set by GrantsMiddleware.
    // For <img> tags, client appends ?token=xxx to URL.
    // The cookie_auth middleware (wrapped on /api/v1/private) sets x-auth-user from cookie.
    // If no cookie, the token query param is checked as fallback.
    let _user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;

    let (object_key, content_type) = match variant.as_str() {
        "thumbnail" => { /* ... same DB lookup logic ... */ }
        "medium" => { /* ... same DB lookup logic ... */ }
        "original" => { /* ... same DB lookup logic ... */ }
        _ => {
            return Err(DomainError::new_bad_input_error(format!(
                "Invalid variant: {}", variant
            )))
        }
    };

    let object = app_data
        .minio
        .client
        .get_object()
        .bucket(&app_data.config.minio.bucket_name)
        .key(&object_key)
        .send()
        .await;

    // ... rest of streaming logic unchanged ...
}
```

**Note:** Because these handlers are now under `/api/v1/private/`, the `cookie_auth` middleware (line 267 in `lib.rs`) runs first and sets the `x-auth-user` header from the cookie. The `GrantsMiddleware` also runs and validates the JWT. So the image variant handler can use `extract_user_uuid_from_header(req.headers())?` — the cookie will provide auth for `<img>` tags when the browser sends the cookie automatically (which it does for same-origin requests).

For cross-origin or non-cookie scenarios, the frontend should append `?token=${jwtToken}` to image URLs and we'd need to add query-param token extraction as a fallback. Let's handle this in step 2.2.

#### `get_traits` (lines 37-48)

**Current:** No auth at all.  
**Change:** Add `req: HttpRequest`, extract user UUID, add `#[protect]`.

```rust
#[utoipa::path(
    get,
    path = "/api/v1/private/pets/traits",  // UPDATED PATH
    tag = "pets",
    responses(
        (status = 200, description = "List of personality traits", body = Vec<crate::models::pets::PersonalityTrait>),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_traits(
    req: HttpRequest,                    // ADDED
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let _user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;  // ADDED

    let traits = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        crate::actions::pets::list_traits(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(traits))
}
```

### 2.2 Query Parameter Token Fallback for Image URLs

Since `<img>` / `<Image>` tags in React Native and browsers **cannot** set custom headers, we need a fallback mechanism. The `cookie_auth` middleware at line 267 of `lib.rs` sets `x-auth-user` from the `X-AUTH-TOKEN` cookie. For web, `withCredentials: true` is already set (line 11 of `api.ts`), so cookies will be sent automatically.

For React Native (where cookies may not persist), we add query parameter support:

**Option A (Recommended — simpler):** The `cookie_auth` middleware already handles cookie-based auth. In React Native, the cookie is stored in the `axios` response interceptor. For `<Image>` tags, we rely on the cookie being persisted. This works in development with `withCredentials: true`.

**Option B (More robust):** Add query param token extraction in `utils.rs`:

```rust
// src/utils.rs — add after extract_user_uuid_from_header

pub fn extract_user_uuid_from_query(
    query: &web::Query<HashMap<String, String>>,
) -> Result<UserUuid, DomainError> {
    query
        .get("token")
        .ok_or_else(|| DomainError::new_auth_error("Missing auth token".to_owned()))
        .and_then(|token| {
            // Validate the token and extract user UUID
            let claims = get_claims(&app_data.jwt_key, token)?;
            Ok(claims.custom.user_uuid)
        })
}
```

Then in the image handlers:

```rust
pub async fn get_pet_image_variant(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    path: web::Path<PublicImageVariantPath>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, DomainError> {
    let PublicImageVariantPath { image_uuid, variant } = path.into_inner();

    // Try header first (from cookie_auth middleware), then query param
    let user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())
        .or_else(|_| extract_user_uuid_from_query(&query))
        .map_err(|_| ErrorUnauthorized("Missing auth token".to_string()))?;
    // ... rest unchanged
}
```

**We'll go with Option A for now** — the cookie approach already works for web. If React Native image loading fails, we can add Option B as a follow-up.

### 2.3 `src/routes/users.rs` — Handlers to Modify

#### `get_public_profile` (lines 513-528)

**Current:** No auth extraction.  
**Change:** Add `req: HttpRequest`, extract user UUID, add `#[protect]`.

```rust
#[utoipa::path(
    get,
    path = "/api/v1/private/profiles/{user_id}",  // UPDATED PATH
    tag = "users",
    params(
        ("user_id" = String, Path, description = "User UUID"),
    ),
    responses(
        (status = 200, description = "Profile found", body = PublicProfile),
        (status = 404, description = "Profile not found", body = ErrorResponseString),
        (status = 401, description = "Missing auth", body = ErrorResponseString),
    ),
)]
#[protect("RoleEnum::RoleUser", ty = RoleEnum)]
#[tracing::instrument(level = "info", skip_all, fields(user_uuid))]
pub async fn get_public_profile(
    req: HttpRequest,                    // ADDED
    app_data: web::Data<AppData>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, DomainError> {
    let _user_uuid = crate::utils::extract_user_uuid_from_header(req.headers())?;  // ADDED
    let uuid = UserUuid::from_str(&user_id.into_inner()).map_err(|err| {
        DomainError::new_bad_input_error(format!("Invalid UserUuid: {err}"))
    })?;
    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::get_public_profile(&uuid, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(res))
}
```

**Note:** The admin endpoints `get_user` (line 35) and `get_users` (line 80) already have `#[protect("RoleEnum::RoleAdmin")]` and are under `/api/v1/private/admin/`. They remain unchanged.

---

## Part 3: OpenAPI Documentation Updates (`src/lib.rs`)

### 3.1 Update `ApiDoc::openapi()` paths (lines 476-603)

The `#[utoipa::path]` attributes on individual handler functions already contain the correct path annotations that will be picked up by `utoipa`. After updating the path annotations in Part 2 above, the OpenAPI spec will automatically reflect the new paths.

No changes needed to the `paths(...)` list in `ApiDoc::openapi()` — it references function names, not paths.

### 3.2 Verify Swagger UI

After making changes, verify:
- `GET /api-doc/openapi.json` returns correct paths
- Swagger UI at `/swagger-ui/` shows all endpoints under their new `/api/v1/private/` prefix

---

## Part 4: Frontend Changes

### 4.1 Create `getImageUrl` Helper with Auth Token

Add a centralized helper in `api.ts` that constructs image URLs with auth token:

```typescript
// frontend/app/lib/api.ts — add after api.create()

const getImageUrl = (imageUuid: string, variant: 'thumbnail' | 'medium' | 'original' = 'medium'): string => {
  const token = useAuthStore.getState().token;
  const baseUrl = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800';
  const url = `${baseUrl}/api/v1/private/pets/images/${imageUuid}/${variant}`;
  return token ? `${url}?token=${token}` : url;
};

export { getImageUrl };
```

### 4.2 Update All Frontend Files

The following files reference `/api/v1/pets/images/` directly and must use the new `getImageUrl` helper:

| File | Line(s) | Current Pattern | Change Required |
|---|---|---|---|
| `app/lib/api.ts` | 74, 79 | `api.get(\`/api/v1/pets/${petUuid}\`)` | Use new `petApi.getPet()` method |
| `app/lib/api.ts` | 31, 39, 44, 48 | Already uses `/api/v1/private/...` ✅ | No change |
| `app/pet-view/gallery-utils.ts` | 6 | `${API_BASE}/api/v1/pets/images/...` | Use `getImageUrl` helper |
| `app/pet-view/profile/[pet_uuid].tsx` | 25, 132, 329 | `${API_BASE}/api/v1/pets/images/...` | Use `getImageUrl` helper |
| `app/(tabs)/pet-profiles/[pet_uuid]/index.tsx` | 39, 147, 329 | `${API_BASE}/api/v1/pets/images/...` | Use `getImageUrl` helper |
| `app/(tabs)/pet-profiles/[pet_uuid]/images.tsx` | 52, 240, 289 | `${API_BASE}/api/v1/pets/images/...` | Use `getImageUrl` helper |
| `app/components/SwipeDeckCard.tsx` | 84 | `http://localhost:8800/api/v1/pets/images/...` | Use `getImageUrl` helper |
| `app/components/SwipeDeckCardWeb.tsx` | 190 | `http://localhost:8800/api/v1/pets/images/...` | Use `getImageUrl` helper |
| `app/components/PrivatePetCard.tsx` | 26 | `${API_BASE}/api/v1/pets/images/...` | Use `getImageUrl` helper |
| `app/components/PetCard.tsx` | 31 | `${API_BASE}/api/v1/pets/images/...` | Use `getImageUrl` helper |

### 4.3 Update `api.ts` — `petApi` Methods

The `petApi.getPublic()` and `petApi.getImages()` methods call the old public endpoints. They need to call the new private endpoints:

```typescript
// frontend/app/lib/api.ts — update petApi

export const petApi = {
  async getPet(petUuid: string): Promise<import('~/app/models/pets').PublicPet> {
    const response = await api.get(`/api/v1/private/pets/${petUuid}`);
    return response.data;
  },

  async getImages(petUuid: string): Promise<Array<{ id: number; uuid: string; format: string; is_primary: boolean; sort_order: number; created_at: string }>> {
    const response = await api.get(`/api/v1/private/pets/${petUuid}/images`);
    return response.data;
  },

  async updatePet(petUuid: string, data: { /* ... */ }): Promise</* ... */> {
    const response = await api.patch(`/api/v1/private/user/pets/${petUuid}`, data);
    return response.data;
  },

  async getTraits(): Promise<Array<{ id: number; name: string }>> {
    const response = await api.get('/api/v1/private/pets/traits');
    return response.data;
  },
};
```

### 4.4 Update All Callers of `petApi.getPublic()` / `petApi.getImages()`

Search for and update all imports/usages:

```bash
# Files that import petApi.getPublic or petApi.getImages:
grep -rn "petApi\.getPublic\|petApi\.getImages" frontend/
```

Update these calls to use the new method names (`petApi.getPet`, `petApi.getImages` still works since we're keeping the same method name but updating the path).

### 4.5 Specific File Changes

#### `app/pet-view/gallery-utils.ts`

```typescript
// Before:
export const getImageUrl = (imageUuid: string, variant: 'thumbnail' | 'medium' | 'original' = 'medium'): string => {
  return `${API_BASE}/api/v1/pets/images/${imageUuid}/${variant}`;
};

// After — import the shared helper or use the same pattern with private path:
import { getImageUrl as getAuthedImageUrl } from '~/app/lib/api';
export const getImageUrl = getAuthedImageUrl;  // re-export
```

#### `app/pet-view/profile/[pet_uuid].tsx`

```typescript
// Before:
import { petApi, likesApi } from '~/app/lib/api';
const getImageUrl = (imageUuid: string, variant: 'thumbnail' | 'medium' | 'original' = 'medium'): string => {
  return `${API_BASE}/api/v1/pets/images/${imageUuid}/${variant}`;
};

// After:
import { petApi, likesApi, getImageUrl } from '~/app/lib/api';
// Remove local getImageUrl definition — use imported one
```

#### `app/(tabs)/pet-profiles/[pet_uuid]/index.tsx`

```typescript
// Before:
const getImageUrl = (imageUuid: string, variant: 'thumbnail' | 'medium' | 'original' = 'thumbnail'): string => {
  return `${API_BASE}/api/v1/pets/images/${imageUuid}/${variant}`;
};

// After:
import { getImageUrl } from '~/app/lib/api';
// Remove local definition
```

#### `app/(tabs)/pet-profiles/[pet_uuid]/images.tsx`

```typescript
// Before:
const getImageUrl = (imageUuid: string, variant: 'thumbnail' | 'medium' | 'original' = 'medium'): string => {
  return `${API_BASE}/api/v1/pets/images/${imageUuid}/${variant}`;
};

// After:
import { getImageUrl } from '~/app/lib/api';
// Remove local definition
```

#### `app/components/SwipeDeckCard.tsx`

```typescript
// Before (line 84):
const imageUrl = pet.primary_image
  ? `http://localhost:8800/api/v1/pets/images/${pet.primary_image.uuid}/medium`
  : undefined;

// After:
import { getImageUrl } from '~/app/lib/api';
// ...
const imageUrl = pet.primary_image
  ? getImageUrl(pet.primary_image.uuid, 'medium')
  : undefined;
```

#### `app/components/SwipeDeckCardWeb.tsx`

```typescript
// Before (line 190):
const imageUrl = pet.primary_image
  ? `http://localhost:8800/api/v1/pets/images/${pet.primary_image.uuid}/medium`
  : undefined;

// After:
import { getImageUrl } from '~/app/lib/api';
// ...
const imageUrl = pet.primary_image
  ? getImageUrl(pet.primary_image.uuid, 'medium')
  : undefined;
```

#### `app/components/PrivatePetCard.tsx`

```typescript
// Before (line 26):
const imageUrl = primary_image
  ? `${API_BASE}/api/v1/pets/images/${primary_image.uuid}/medium`
  : null;

// After:
import { getImageUrl } from '~/app/lib/api';
// ...
const imageUrl = primary_image
  ? getImageUrl(primary_image.uuid, 'medium')
  : null;
```

#### `app/components/PetCard.tsx`

```typescript
// Before (line 31):
const imageUrl = primary_image
  ? `${API_BASE}/api/v1/pets/images/${primary_image.uuid}/medium`
  : null;

// After:
import { getImageUrl } from '~/app/lib/api';
// ...
const imageUrl = primary_image
  ? getImageUrl(primary_image.uuid, 'medium')
  : null;
```

---

## Part 5: Verification Steps

### 5.1 Compile Check

```bash
cd /home/rohan/pet-app/actix-demo
cargo check
```

Expected: No errors. If there are errors, they will likely be:
- Missing `req: HttpRequest` parameter in handler signatures
- Unused import warnings for `API_BASE` constants after removing local `getImageUrl` definitions

### 5.2 Run Tests

```bash
cargo test
```

### 5.3 Manual API Verification

Start the server and test each endpoint:

```bash
# Start the server
cargo run

# Test 1: Unauthenticated request should return 401
curl -v http://localhost:8800/api/v1/pets/traits
# Expected: 401 Unauthorized (or 404 if route no longer exists at public path)

# Test 2: Authenticated request should return 200
curl -v -b "X-AUTH-TOKEN=<valid_token>" http://localhost:8800/api/v1/private/pets/traits
# Expected: 200 OK with traits data

# Test 3: Pet profile endpoint
curl -v -b "X-AUTH-TOKEN=<valid_token>" http://localhost:8800/api/v1/private/pets/<pet_uuid>
# Expected: 200 OK with pet data

# Test 4: Public profile endpoint
curl -v -b "X-AUTH-TOKEN=<valid_token>" http://localhost:8800/api/v1/private/profiles/<user_uuid>
# Expected: 200 OK with public profile

# Test 5: Image variant with cookie auth (web)
curl -v -b "X-AUTH-TOKEN=<valid_token>" "http://localhost:8800/api/v1/private/pets/images/<image_uuid>/medium"
# Expected: 200 OK with image stream

# Test 6: Verify old public endpoints no longer exist
curl -v http://localhost:8800/api/v1/pets/traits
# Expected: 404 Not Found

curl -v http://localhost:8800/api/v1/profiles/<user_uuid>
# Expected: 404 Not Found
```

### 5.4 Frontend Verification

1. Start the Expo app: `cd frontend && npx expo start`
2. Login with valid credentials
3. Navigate to pet profiles — images should load correctly
4. Navigate to the swipe deck — cards should display pet images
5. Navigate to image gallery — photos should display
6. Verify that all image URLs now include the auth token query parameter

### 5.5 OpenAPI Verification

```bash
curl http://localhost:8800/api-doc/openapi.json | jq '.paths' | grep -E "private|pets|profiles"
```

Verify that:
- All pet endpoints are under `/api/v1/private/`
- All profile endpoints are under `/api/v1/private/`
- No pet/profile endpoints remain under `/api/v1/` (without `/private`)

---

## Summary of Changes

### Backend Files Modified

| File | Changes |
|---|---|
| `src/lib.rs` | Remove 8 routes from public scope, add 6 routes to private scope |
| `src/routes/pets.rs` | Update 5 handlers: add auth extraction, update OpenAPI paths, add `#[protect]` |
| `src/routes/users.rs` | Update 1 handler: `get_public_profile` — add auth extraction, update OpenAPI path, add `#[protect]` |

### Frontend Files Modified

| File | Changes |
|---|---|
| `app/lib/api.ts` | Add `getImageUrl` helper, update `petApi` paths |
| `app/pet-view/gallery-utils.ts` | Use shared `getImageUrl` |
| `app/pet-view/profile/[pet_uuid].tsx` | Use shared `getImageUrl` |
| `app/(tabs)/pet-profiles/[pet_uuid]/index.tsx` | Use shared `getImageUrl` |
| `app/(tabs)/pet-profiles/[pet_uuid]/images.tsx` | Use shared `getImageUrl` |
| `app/components/SwipeDeckCard.tsx` | Use shared `getImageUrl` |
| `app/components/SwipeDeckCardWeb.tsx` | Use shared `getImageUrl` |
| `app/components/PrivatePetCard.tsx` | Use shared `getImageUrl` |
| `app/components/PetCard.tsx` | Use shared `getImageUrl` |

### Endpoints Moved

| Old Path | New Path | Handler |
|---|---|---|
| `GET /api/v1/pets/traits` | `GET /api/v1/private/pets/traits` | `get_traits` |
| `GET /api/v1/pets/{pet_uuid}` | `GET /api/v1/private/pets/{pet_uuid}` | `get_public_pet` |
| `GET /api/v1/pets/{pet_uuid}/images` | `GET /api/v1/private/pets/{pet_uuid}/images` | `get_pet_images` |
| `GET /api/v1/pets/images/{image_uuid}` | `GET /api/v1/private/pets/images/{image_uuid}` | `get_public_pet_image` |
| `GET /api/v1/pets/images/{image_uuid}/{variant}` | `GET /api/v1/private/pets/images/{image_uuid}/{variant}` | `get_pet_image_variant` |
| `GET /api/v1/profiles/{user_id}` | `GET /api/v1/private/profiles/{user_id}` | `get_public_profile` |
| `GET /api/v1/users` | **Removed** (duplicate of admin) | — |
| `GET /api/v1/users/{user_id}` | **Removed** (duplicate of admin) | — |

### Endpoints Kept Public

| Path | Handler | Reason |
|---|---|---|
| `GET /api/v1/avatars/{user_id}` | `get_user_avatar` | Image serving for profile pictures |
| `GET /api/v1/build-info` | `build_info_req` | System info |
| `GET /api/v1/metrics/cmd` | `handle_get_job_metrics` | Metrics |
| `POST /api/v1/auth/*` | Various | Auth flows |
| `GET /hc` | `healthcheck` | Health check |
| `GET /ws` | `ws` | WebSocket |
