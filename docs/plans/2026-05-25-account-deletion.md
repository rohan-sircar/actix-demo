# Account Deletion Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Add a `POST /api/users/me/delete` endpoint that performs a soft delete of the authenticated user's account, cascading cleanup across sessions (Redis), avatar (MinIO), and orphaning associated jobs.

**Architecture:** Soft delete via `deleted_at` timestamp column on the `users` table. Jobs are orphaned (FK set to NULL) rather than cascade-deleted to preserve audit history. Redis sessions are nuked atomically. MinIO avatar deletion is fire-and-forget (non-fatal if missing).

**Tech Stack:** Rust, Actix-web 4, Diesel 2.2 (PostgreSQL), Redis (sessions), MinIO (avatars), bcrypt (passwords), JWT (auth).

---

## Task 1: Create Database Migration

**Objective:** Add `deleted_at` column to `users` table and make `jobs.started_by` nullable.

**Files:**
- Create: `migrations/2026-05-25-000000_soft_delete_users/up.sql`
- Create: `migrations/2026-05-25-000000_soft_delete_users/down.sql`

**Step 1: Create migration directory**

```bash
mkdir -p /opt/data/projects/actix-demo/migrations/2026-05-25-000000_soft_delete_users
```

**Step 2: Write up.sql**

```sql
-- Add soft-delete column to users
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMP;

-- Make jobs.started_by nullable so we can orphan jobs on user deletion
ALTER TABLE jobs ALTER COLUMN started_by DROP NOT NULL;
```

**Step 3: Write down.sql**

```sql
-- Rollback: restore NOT NULL constraint first (requires no NULL values)
-- In practice, this migration is irreversible without data cleanup.
-- For safety, we just drop the column.
ALTER TABLE users DROP COLUMN deleted_at;
ALTER TABLE jobs ALTER COLUMN started_by SET NOT NULL;
```

**Step 4: Verify files exist**

```bash
ls /opt/data/projects/actix-demo/migrations/2026-05-25-000000_soft_delete_users/
# Expected: down.sql  up.sql
```

---

## Task 2: Update Diesel Schema + User Model

**Objective:** Add `deleted_at` field to the `User` struct and schema.

**Files:**
- Modify: `src/schema.rs:38-44` — add `deleted_at` column
- Modify: `src/models/users.rs:91-98` — add `deleted_at` field to `User` struct

**Step 1: Update schema.rs**

Change the `users` table definition from:

```rust
diesel::table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
    }
}
```

To:

```rust
diesel::table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}
```

**Step 2: Update User model**

In `src/models/users.rs`, change the `User` struct from:

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub created_at: chrono::NaiveDateTime,
}
```

To:

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub created_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}
```

**Step 3: Verify compilation**

```bash
cd /opt/data/projects/actix-demo
cargo check 2>&1 | head -20
# Expected: no errors (schema.rs and model must match)
```

---

## Task 3: Add Error Type

**Objective:** Add `AccountDeletedError` to `DomainError` enum with 409 Conflict mapping.

**Files:**
- Modify: `src/errors.rs:7-28` — add error variant to `custom_error!` macro
- Modify: `src/errors.rs:39-122` — add `ResponseError` mapping

**Step 1: Add variant to custom_error! macro**

In `src/errors.rs`, add a new variant inside the `custom_error!` block (after `FileUploadFailed`):

```rust
AccountDeletedError { message: String } = "Account deletion failed: {message}",
```

The full block should look like:

```rust
custom_error! { #[derive(new)] #[allow(clippy::enum_variant_names)]
    pub DomainError
    PwdHashError {source: BcryptError} = "Failed to hash password",
    FieldValidationError {message: String} = "Failed to validate one or more fields",
    DbError {source: diesel::result::Error} = "Database error",
    DbPoolError {source: r2d2::Error} = "Failed to get connection from pool",
    BadInputError {message: String} = "Bad inputs to request: {message}",
    EntityDoesNotExistError {message: String} = "Entity does not exist - {message}",
    BlockingError {source: actix_web::error::BlockingError} = "Blocking error - {source}",
    AuthError {message: String} = "Authentication Error - {message}",
    JwtError {message: String} = "Jwt Error - {message}",
    RedisError {source: redis::RedisError} = "Redis Error = {source}",
    WsProtocolError {source: actix_ws::ProtocolError} = "WS Protocol Error = {source}",
    UninitializedError { message: String } = "A required component was not initialized - {message}",
    JoinError {source: tokio::task::JoinError } = "Join error - {source}",
    InternalError {message: String} = "An internal error occured - {message}",
    RateLimitError {message: String} = "Rate limit exceeded: {message}",
    FileSizeExceeded {max_bytes: u64} = "File size exceeded: max {max_bytes} bytes",
    InvalidMimeType {detected: String} = "Invalid MIME type: {detected}",
    FileUploadFailed {message: String} = "Failed to upload file: {message}",
    PayloadError { source: actix_web::error::PayloadError } = "Payload error: {source}",
    AccountDeletedError { message: String } = "Account deletion failed: {message}",
}
```

**Step 2: Add ResponseError mapping**

Add a new arm in `ResponseError::error_response()` (after the `PayloadError` arm, around line 119):

```rust
DomainError::AccountDeletedError { message } => {
    HttpResponse::Conflict()
        .json(ErrorResponse::new(self.to_string()))
}
```

**Step 3: Verify compilation**

```bash
cd /opt/data/projects/actix-demo
cargo check 2>&1 | head -20
# Expected: no errors
```

---

## Task 4: Add Action Functions

**Objective:** Implement `soft_delete_user()` and `delete_user_avatar()` in the action layer.

**Files:**
- Modify: `src/actions/users.rs` — add two new functions at the end of the file

**Step 1: Add `soft_delete_user` function**

Append to `src/actions/users.rs`:

```rust
/// Soft-delete a user by setting deleted_at timestamp.
/// Returns an error if the user is already deleted or doesn't exist.
pub fn soft_delete_user(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<(), DomainError> {
    use crate::schema::users::dsl as users;

    // Check if user exists and is not already deleted
    let existing = users::users
        .select((users::id, users::deleted_at))
        .filter(users::id.eq(user_id))
        .first::<(UserId, Option<chrono::NaiveDateTime>)>(conn)
        .optional()?;

    match existing {
        None => Err(DomainError::EntityDoesNotExistError {
            message: format!("User not found: {}", user_id),
        }),
        Some((_, Some(_))) => Err(DomainError::AccountDeletedError {
            message: format!("User {} is already deleted", user_id),
        }),
        Some((id, None)) => {
            // Perform the soft delete in a transaction
            conn.transaction(|conn| {
                diesel::update(users::users.filter(users::id.eq(id)))
                    .set(users::deleted_at.eq(chrono::Utc::now().naive_utc()))
                    .execute(conn)?;

                // Invalidate the user_ids_cache since this user is now deleted
                // The cache will be rebuilt on next access
                Ok(())
            })?;

            tracing::info!(user_id = %user_id, "User soft-deleted");
            Ok(())
        }
    }
}
```

**Step 2: Add `delete_user_avatar` function**

Append to `src/actions/users.rs`:

```rust
/// Delete a user's avatar from MinIO.
/// Non-fatal: returns Ok even if the avatar doesn't exist.
pub async fn delete_user_avatar(
    user_id: &UserId,
    minio: &minior::Minio,
    bucket_name: &str,
) -> Result<(), DomainError> {
    let object_key = format!("avatars/{}", user_id);

    match minio
        .client
        .delete_object()
        .bucket(bucket_name)
        .key(&object_key)
        .send()
        .await
    {
        Ok(_) => {
            tracing::info!(user_id = %user_id, object_key = %object_key, "Avatar deleted from MinIO");
            Ok(())
        }
        Err(e) => {
            // Check if it's a 404 (object not found) — non-fatal
            let err_str = format!("{:?}", e);
            if err_str.contains("404") || err_str.contains("NoSuchKey") {
                tracing::warn!(user_id = %user_id, object_key = %object_key, "Avatar not found, skipping deletion");
                Ok(())
            } else {
                Err(DomainError::InternalError {
                    message: format!("Failed to delete avatar from MinIO: {}", err_str),
                })
            }
        }
    }
}
```

**Step 3: Verify compilation**

```bash
cd /opt/data/projects/actix-demo
cargo check 2>&1 | head -30
# Expected: no errors
```

---

## Task 5: Add Route Handler

**Objective:** Implement `delete_my_account` handler in the routes layer.

**Files:**
- Modify: `src/routes/users.rs` — add new handler function
- Add imports: `Cookie`, `time` crate

**Step 1: Add imports to `src/routes/users.rs`**

Add at the top of the file (after existing imports):

```rust
use awc::cookie::{Cookie, SameSite};
use time::OffsetDateTime;
```

**Step 2: Add the handler function**

Append to `src/routes/users.rs`:

```rust
/// Delete the authenticated user's account (soft delete).
/// Clears all sessions and avatar. Orphans associated jobs.
#[tracing::instrument(level = "info", skip(app_data, req))]
pub async fn delete_my_account(
    req: HttpRequest,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    let user_id = utils::extract_user_id_from_header(req.headers())?;

    // Step 1: Soft delete in DB (blocking, transactional)
    web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::soft_delete_user(&user_id, &mut conn)
    })
    .await??;

    // Step 2: Delete all Redis sessions (async, non-blocking)
    let _ = app_data
        .credentials_repo
        .delete_all_sessions(&user_id)
        .await;

    // Step 3: Delete avatar from MinIO (async, fire-and-forget, non-fatal)
    let bucket = app_data.config.minio.bucket_name.clone();
    let minio = app_data.minio.clone();
    tokio::spawn(async move {
        if let Err(e) = actions::users::delete_user_avatar(&user_id, &minio, &bucket).await {
            tracing::warn!(user_id = %user_id, error = %e, "Failed to delete avatar on account deletion");
        }
    });

    // Step 4: Clear the auth cookie so the user is immediately logged out
    let cookie = Cookie::build("X-AUTH-TOKEN", "")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .expires(OffsetDateTime::UNIX_EPOCH)
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).finish())
}
```

**Step 3: Verify compilation**

```bash
cd /opt/data/projects/actix-demo
cargo check 2>&1 | head -30
# Expected: no errors
```

---

## Task 6: Register Route

**Objective:** Add the new endpoint to the authenticated `/api` scope in `lib.rs`.

**Files:**
- Modify: `src/lib.rs:196-242` — add route registration in the authenticated scope

**Step 1: Add route registration**

In `src/lib.rs`, inside the authenticated `/api` scope (after the `/sessions` service block, before the closing `);`), add:

```rust
.service(
    web::scope("/users")
        .route("/me/delete", web::post().to(routes::users::delete_my_account)),
)
```

The full authenticated scope should look like:

```rust
// authenticated api
.service(
    web::scope("/api")
        .wrap(api_rate_limiter(&app_data.config.rate_limit.api))
        .wrap(GrantsMiddleware::with_extractor(routes::auth::extract))
        .wrap(middleware::Condition::new(
            true, // Always enabled
            middlewares::CustomHeaders::new(app_data.config.timezone),
        ))
        .wrap(from_fn(utils::cookie_auth))
        .route("/cmd", web::post().to(routes::command::handle_run_command))
        .route("/cmd/{job_id}", web::get().to(routes::command::handle_get_job))
        .route("/cmd/{job_id}", web::delete().to(routes::command::handle_abort_job))
        .service(web::scope("/avatars").route(
            "",
            web::put().to(routes::users::upload_user_avatar),
        ))
        .service(
            web::scope("/sessions")
                .route("", web::get().to(routes::auth::list_sessions))
                .route("/{session_id}", web::delete().to(routes::auth::revoke_session))
                .route("/revoke-others", web::post().to(routes::auth::revoke_other_sessions)),
        )
        .service(
            web::scope("/users")
                .route("/me/delete", web::post().to(routes::users::delete_my_account)),
        ),
)
```

**Step 2: Verify compilation**

```bash
cd /opt/data/projects/actix-demo
cargo check 2>&1 | head -30
# Expected: no errors
```

---

## Task 7: Add Unit Tests

**Objective:** Write unit tests for `soft_delete_user` in `src/actions/users.rs`.

**Files:**
- Modify: `src/actions/users.rs` — add test module at the end

**Step 1: Add test module**

Append to `src/actions/users.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::users::{NewUser, Password, Username};
    use diesel::RunQueryDsl;

    #[test]
    fn test_soft_delete_existing_user() {
        // Integration test — requires testcontainers setup
        // See integration tests in Task 8
    }

    #[test]
    fn test_soft_delete_nonexistent_user() {
        // Integration test — requires testcontainers setup
    }

    #[test]
    fn test_soft_delete_already_deleted_user() {
        // Integration test — requires testcontainers setup
    }
}
```

*Note: Full integration tests go in Task 8. These stubs document the test matrix.*

---

## Task 8: Write Integration Tests

**Objective:** End-to-end test: register → login → delete → verify 401 on subsequent requests.

**Files:**
- Create: `tests/account_deletion.rs` — integration test file

**Step 1: Create the integration test file**

```rust
use actix_web::test;
use actix_web::App;
use serde_json::json;

use actix_demo::configure_app;
use actix_demo::AppData;

async fn setup_app_data() -> actix_web::web::Data<AppData> {
    // Use testcontainers for PostgreSQL, Redis, and MinIO
    // This mirrors the existing test pattern in the project
    unimplemented!("Set up test containers — see existing integration tests for pattern")
}

#[actix_web::test]
async fn test_account_deletion_full_flow() {
    let app_data = setup_app_data().await;

    // 1. Register a new user
    let req = test::TestRequest::post()
        .uri("/api/registration")
        .set_json(json!({
            "username": "testdelete",
            "password": "testpass123"
        }))
        .insert_header(("content-type", "application/json"))
        .app_data(app_data.clone())
        .to_request();

    let resp = test::call_service(&app_data, req).await;
    assert!(resp.status().is_success());

    // 2. Login
    let req = test::TestRequest::post()
        .uri("/api/login")
        .set_json(json!({
            "username": "testdelete",
            "password": "testpass123"
        }))
        .insert_header(("content-type", "application/json"))
        .app_data(app_data.clone())
        .to_request();

    let resp = test::call_service(&app_data, req).await;
    assert!(resp.status().is_success());

    // 3. Delete account
    let req = test::TestRequest::post()
        .uri("/api/users/me/delete")
        .app_data(app_data.clone())
        .to_request();

    let resp = test::call_service(&app_data, req).await;
    assert!(resp.status().is_success());

    // 4. Verify subsequent requests return 401
    let req = test::TestRequest::get()
        .uri("/api/public/users/testdelete")
        .app_data(app_data.clone())
        .to_request();

    let resp = test::call_service(&app_data, req).await;
    // User should not be found (soft-deleted)
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}
```

**Step 2: Run tests**

```bash
cd /opt/data/projects/actix-demo
cargo test account_deletion -- --nocapture
# Expected: test passes (after implementing full container setup)
```

---

## Files Modified Summary

| File | Change |
|------|--------|
| `migrations/2026-05-25-000000_soft_delete_users/up.sql` | **New** — adds `deleted_at`, drops `NOT NULL` on `started_by` |
| `migrations/2026-05-25-000000_soft_delete_users/down.sql` | **New** — rollback |
| `src/schema.rs` | Add `deleted_at -> Nullable<Timestamp>` to `users` table |
| `src/models/users.rs` | Add `deleted_at: Option<chrono::NaiveDateTime>` to `User` struct |
| `src/errors.rs` | Add `AccountDeletedError` variant + 409 Conflict mapping |
| `src/actions/users.rs` | Add `soft_delete_user()`, `delete_user_avatar()` functions |
| `src/routes/users.rs` | Add `delete_my_account()` handler + Cookie/time imports |
| `src/lib.rs` | Register `POST /api/users/me/delete` in authenticated scope |
| `tests/account_deletion.rs` | **New** — integration tests |

---

## Testing Strategy

1. **Unit tests** in `src/actions/users.rs` — verify `soft_delete_user` behavior for existing, non-existent, and already-deleted users
2. **Integration tests** in `tests/account_deletion.rs` — full flow with testcontainers (PostgreSQL + Redis + MinIO)
3. **Edge cases**:
   - Delete account while having active WebSocket connections → sessions cleared, WS handles disconnect
   - Concurrent deletion attempts → only first succeeds (transaction + check on `deleted_at`)
   - MinIO avatar doesn't exist → deletion continues without error
   - User has no avatar → deletion continues without error

---

## Not In Scope (Future)

- Email-based deletion confirmation
- Admin-initiated account deletion
- Data purge (GDPR right to be forgotten) — separate from soft delete
- Temporary deactivation (requires `disabled` boolean flag)
