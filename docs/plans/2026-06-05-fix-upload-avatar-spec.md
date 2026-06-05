# Fix: Upload Avatar OpenAPI Spec

## Problem
`upload_user_avatar` currently has no `request_body` annotation. The handler uses manual `web::Payload` multipart parsing, so utoipa's `actix_extras` auto-detection can't pick it up. The generated spec shows the POST `/api/avatars` endpoint with no request body.

## Steps

### 1. Add DTO struct in `src/routes/users.rs`

Add this after the `UpdateUserProfile` definition (around line 200):

```rust
/// Request body for avatar upload.
#[derive(ToSchema)]
pub struct UploadAvatarRequest {
    /// Avatar image file (PNG, JPG, GIF, WebP).
    pub file: Vec<u8>,
}
```

### 2. Update the `#[utoipa::path]` annotation on `upload_user_avatar`

Replace the existing annotation (currently has no `request_body`):

```rust
#[utoipa::path(
    put,
    path = "/api/avatars",
    tag = "users",
    request_body = UploadAvatarRequest,
    content_type = "multipart/form-data",
    responses(
        (status = 200, description = "Avatar uploaded successfully", body = String),
        (status = 400, description = "Invalid file type or size", body = crate::models::misc::ErrorResponse<String>),
        (status = 401, description = "Missing or invalid auth token", body = crate::models::misc::ErrorResponse<String>),
    ),
)]
```

### 3. Register `UploadAvatarRequest` in `ApiDoc` components

In `src/lib.rs`, add `routes::users::UploadAvatarRequest` to the `schemas(...)` list inside `#[openapi(components(schemas(...)))]`.

### 4. Verify

Run `cargo check` to ensure it compiles. The generated spec at `/api/docs` should show `POST /api/avatars` with a multipart/form-data request body containing a `file` field of type `string, format: binary`.
