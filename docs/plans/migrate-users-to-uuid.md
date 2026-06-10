# Plan: Migrate User APIs to Use UUIDs

## Scope Summary

Add a `user_uuid` column to the `users` table. Keep `UserId` (i32) for internal/database operations (foreign keys, joins). Use `UserUuid` for all public-facing API lookups, JWT claims, and response serialization.

## Files to Modify

| # | File | Changes |
|---|------|---------|
| 1 | `src/models/users.rs` | Add `UserUuid` newtype; add `user_uuid` field to all user models |
| 2 | `src/schema.rs` | Add `user_uuid -> Uuid` to `users` table definition |
| 3 | `migrations/2026-XX-XX-add_user_uuid_to_users/up.sql` | New migration: add column, set default, backfill, add unique index |
| 4 | `src/routes/auth.rs` | Store `user_uuid` in JWT claims; update `extract()` to use UUID |
| 5 | `src/utils.rs` | Update `extract_user_id_from_header()` to parse UUID |
| 6 | `src/utils/redis_credentials_repo.rs` | Change Redis key prefix from `{user_id}` to `{user_uuid}` |
| 7 | `src/routes/users.rs` | Change path params from `UserId` to `UserUuid`; update all lookups |
| 8 | `src/actions/users.rs` | Change lookup functions to accept `&UserUuid`; resolve to `&UserId` for DB ops |
| 9 | `src/models/misc.rs` | Update `Job.started_by` type consideration (keep as UserId internally) |
| 10 | `src/models/ws.rs` | Update `WsClientEvent.SendMessage.receiver` and `SentMessage.sender` |
| 11 | `tests/integration/users.rs` | Update assertions to use UUIDs from responses |
| 12 | `tests/integration/common/mod.rs` | Update test helpers that reference user IDs |
| 13 | `tests/integration/auth/*.rs` | Update any auth tests referencing user IDs |

## Detailed Steps

### Step 1: Add `UserUuid` type (`src/models/users.rs`)

Create a newtype wrapping `uuid::Uuid`, following the `PetUuid` pattern:

```rust
#[derive(
    Debug, Clone, Eq, Hash, PartialEq, Deserialize, Display, Into, Serialize,
    DieselNewType, Copy, ToSchema,
)]
#[serde(try_from = "String", into = "String")]
pub struct UserUuid(Uuid);

impl UserUuid {
    pub fn as_uuid(&self) -> Uuid { self.0 }
}

impl From<UserUuid> for String {
    fn from(s: UserUuid) -> String { s.0.to_string() }
}

impl FromStr for UserUuid {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(UserUuid).map_err(|e| format!("invalid UUID format: {}", e))
    }
}

impl TryFrom<String> for UserUuid {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse::<UserUuid>()
    }
}
```

### Step 2: Update models to include `user_uuid`

Add `pub user_uuid: UserUuid` to:
- `User` model (Queryable + Serialize + ToSchema) — **add to select queries**
- `UserWithRoles` model
- `UserAuthDetails` model
- `UserAuthDetailsWithRoles` model
- `OAuthUserLookup` model

`NewUser` and `UpdateUserProfile` do NOT need `user_uuid` (generated on insert, not updatable).

### Step 3: Database migration

Create `migrations/2026-06-09-000000_add_user_uuid_to_users/`:
- `up.sql`: Add `user_uuid UUID NOT NULL DEFAULT gen_random_uuid()` to users table; add `UNIQUE` index on `user_uuid`; backfill existing rows
- `down.sql`: Drop index, drop column

Then run `diesel print-schema` to update `src/schema.rs`.

**up.sql:**
```sql
ALTER TABLE users ADD COLUMN user_uuid UUID NOT NULL DEFAULT gen_random_uuid();
CREATE UNIQUE INDEX users_user_uuid_idx ON users(user_uuid);
UPDATE users SET user_uuid = gen_random_uuid() WHERE user_uuid IS NULL;
```

**down.sql:**
```sql
DROP INDEX IF EXISTS users_user_uuid_idx;
ALTER TABLE users DROP COLUMN IF EXISTS user_uuid;
```

### Step 4: Update JWT claims (`src/routes/auth.rs`)

Change `VerifiedAuthDetails`:
```rust
pub struct VerifiedAuthDetails {
    pub user_uuid: UserUuid,  // was: user_id: UserId
    pub session_id: Uuid,
    pub username: Username,
    pub roles: Vec<RoleEnum>,
    pub device_id: String,
}
```

Update `login()` to store `user.user_uuid` in claims. Update `extract()` to read `claims.custom.user_uuid` and set `x-auth-user` header with UUID string. Update `validate_token()` and `logout()` to use `user_uuid`.

### Step 5: Update header extraction (`src/utils.rs`)

Rename `extract_user_id_from_header` → `extract_user_uuid_from_header()`, change return type to `UserUuid`:

```rust
pub fn extract_user_uuid_from_header(headers: &HeaderMap) -> Result<UserUuid, DomainError> {
    extract_header_value(headers, "x-auth-user").and_then(|s| {
        UserUuid::from_str(&s).map_err(|err| {
            DomainError::new_bad_input_error(format!(
                "Invalid UserUuid format in x-auth-user header: {err}"
            ))
        })
    })
}
```

Update all call sites.

### Step 6: Update Redis credentials repo (`src/utils/redis_credentials_repo.rs`)

Change all methods to accept `&UserUuid` instead of `&UserId`:
- `get_key(&self, user_uuid: &UserUuid)`
- `get_expiry_key(&self, user_uuid: &UserUuid, ...)`
- `is_token_expired`, `load_session`, `load_all_sessions`, `create_session`, etc.

Update Redis key format: `app.user-sessions.{uuid}` instead of `app.user-sessions.{int_id}`.

### Step 7: Update routes (`src/routes/users.rs`)

Change all path params from `web::Path<UserId>` to `web::Path<UserUuid>`:
- `get_user` — path param becomes UUID
- `get_user_avatar` — path param becomes UUID, MinIO key uses UUID
- `get_public_profile` — path param becomes UUID

Update `add_user` route to return user with `user_uuid`. Update email verification token insertion to resolve UUID → ID for the `user_id` FK column.

### Step 8: Update actions (`src/actions/users.rs`)

Each lookup function now accepts `&UserUuid` and queries by `user_uuid`:

```rust
pub fn find_user_by_uuid(uid: &UserUuid, conn: &mut DbConnection) -> Result<Option<UserWithRoles>, DomainError> {
    use crate::schema::users::dsl as users;
    conn.transaction(|conn| {
        let mb_user = users::users
            .select((users::id, users::username, users::created_at, users::deleted_at))
            .filter(users::user_uuid.eq(uid))  // query by UUID
            .first::<User>(conn).optional()?;
        let roles = mb_user.as_ref().map(|u| get_roles_for_user(&u.id, conn)).transpose()?;
        Ok(mb_user.map(|user| UserWithRoles::from_user(&user, &roles.unwrap_or_default())))
    })
}
```

Functions to update:
- `find_user_by_uid` → `find_user_by_uuid` (accepts `&UserUuid`, queries by `user_uuid`)
- `find_active_user_by_uid` → `find_active_user_by_uuid`
- `get_all_user_ids` — keep as-is (internal cache, uses integer ID)
- `soft_delete_user` — accept `&UserUuid`, resolve to ID for FK ops (jobs.started_by)
- `delete_user_avatar` — accept `&UserUuid`
- `update_user_profile` — accept `&UserUuid`, resolve to ID for FK updates
- `find_or_create_oauth_user` — return UUID in result

For functions that need the integer ID for FK operations (e.g., soft delete clearing jobs), resolve UUID → ID first via the User model's `id` field after lookup.

### Step 9: Update WebSocket models (`src/models/ws.rs`)

Change:
- `WsClientEvent.SendMessage.receiver` from `UserId` to `UserUuid`
- `SentMessage.sender` from `UserId` to `UserUuid`

### Step 10: Update `PublicProfile` (`src/models/users.rs`)

Change `user_id: UserId` to `user_uuid: UserUuid` in `PublicProfile` struct. Update `From<&Profile>` impl.

### Step 11: Update tests

- `tests/integration/users.rs`: Replace assertions like `assert_eq!(user.id.as_uint(), 1)` with UUID-based checks (extract UUID from response body)
- `tests/integration/common/mod.rs`: Update test helpers that construct or reference user IDs
- `tests/integration/auth/session/*.rs`: Update session tests using user IDs
- `tests/integration/auth/registration_verification_login.rs`: Update registration tests
- `tests/integration/auth/email_verification.rs`: Update email verification tests
- `tests/integration/misc.rs`: Check for any user ID references

## Key Design Decisions

1. **`UserId` stays for DB internals** — Foreign keys in `profiles`, `pets`, `users_roles`, `jobs`, `email_verification_tokens`, `password_reset_tokens` remain `Int4`. Only the public API surface uses UUIDs.

2. **`user_uuid` is generated on insert** — The column has `DEFAULT gen_random_uuid()`. `NewUser` insertable does NOT include `user_uuid`.

3. **JWT stores UUID string** — The custom claims store `user_uuid` as a string. This replaces the integer `user_id` in JWT claims.

4. **Resolution pattern** — When a UUID is received from the API, it's used directly for user lookups. When integer ID is needed for FK operations (e.g., `jobs.started_by`), a lookup resolves `user_uuid → user_id`.

5. **Breaking API change** — All client code using integer user IDs will need to switch to UUIDs. This is intentional.

6. **Response models include both** — `User` and `UserWithRoles` will have both `id: UserId` (internal) and `user_uuid: UserUuid` (public). The OpenAPI schema will expose `user_uuid` as the primary identifier.

## Order of Operations

1. Create migration and run it
2. Run `diesel print-schema` to update schema.rs
3. Add `UserUuid` type to models/users.rs
4. Add `user_uuid` field to model structs
5. Update JWT claims (routes/auth.rs)
6. Update header extraction (utils.rs)
7. Update Redis credentials repo
8. Update actions (lookup functions)
9. Update routes (path params, responses)
10. Update WebSocket models
11. Update PublicProfile response
12. Update all tests
13. Run `cargo make lint-check` to verify
