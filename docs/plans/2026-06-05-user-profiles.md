# User Profile System

## Overview

Add a user profile system with public profiles, editable fields (bio, display name, location, website, social links), and OpenAPI schema support.

## Database Migration

**File:** `migrations/0001_create_profiles.sql`

```sql
CREATE TABLE profiles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    bio TEXT,
    display_name VARCHAR(100),
    location VARCHAR(200),
    website_url VARCHAR(500),
    social_github VARCHAR(100),
    social_twitter VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE OR REPLACE FUNCTION update_profiles_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_profiles_updated_at
    BEFORE UPDATE ON profiles
    FOR EACH ROW EXECUTE FUNCTION update_profiles_updated_at();
```

## Files to Create/Modify

### 1. `src/schema.rs` — add profiles table definition

```rust
diesel::table! {
    use diesel::sql_types::*;

    profiles (id) {
        id -> Int4,
        user_id -> Int4,
        bio -> Nullable<Text>,
        #[max_length = 100]
        display_name -> Nullable<Varchar>,
        #[max_length = 200]
        location -> Nullable<Varchar>,
        #[max_length = 500]
        website_url -> Nullable<Varchar>,
        #[max_length = 100]
        social_github -> Nullable<Varchar>,
        #[max_length = 100]
        social_twitter -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(profiles -> users (user_id));
```

### 2. `src/models/users.rs` — add newtype wrappers + models

Newtypes (all `DieselNewType` for Diesel compatibility):

| Newtype | Constraint | Diesel type |
|---------|-----------|-------------|
| `Bio` | line max 500 chars | Text |
| `DisplayName` | line 1-100 chars | Varchar(100) |
| `Location` | line max 200 chars | Varchar(200) |
| `WebsiteUrl` | url validator | Varchar(500) |
| `SocialGithub` | GitHub handle regex (alphanumeric + hyphens, 1-39 chars) | Varchar(100) |
| `SocialTwitter` | Twitter handle regex (alphanumeric + underscore, 1-15 chars) | Varchar(100) |

Models:

```rust
#[derive(Debug, Clone, Queryable, ToSchema)]
#[diesel(table_name = profiles)]
pub struct Profile {
    pub id: i32,
    pub user_id: UserId,
    pub bio: Option<Bio>,
    pub display_name: Option<DisplayName>,
    pub location: Option<Location>,
    pub website_url: Option<WebsiteUrl>,
    pub social_github: Option<SocialGithub>,
    pub social_twitter: Option<SocialTwitter>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Deserialize, ToSchema)]
#[diesel(table_name = profiles)]
pub struct NewProfile {
    pub user_id: UserId,
    pub bio: Option<Bio>,
    pub display_name: Option<DisplayName>,
    pub location: Option<Location>,
    pub website_url: Option<WebsiteUrl>,
    pub social_github: Option<SocialGithub>,
    pub social_twitter: Option<SocialTwitter>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, ToSchema)]
#[serde(default)]
pub struct UpdateProfile {
    pub bio: Option<Bio>,
    pub display_name: Option<DisplayName>,
    pub location: Option<Location>,
    pub website_url: Option<WebsiteUrl>,
    pub social_github: Option<SocialGithub>,
    pub social_twitter: Option<SocialTwitter>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PublicProfile {
    pub user_id: UserId,
    pub bio: Option<Bio>,
    pub display_name: Option<DisplayName>,
    pub location: Option<Location>,
    pub website_url: Option<WebsiteUrl>,
    pub social_github: Option<SocialGithub>,
    pub social_twitter: Option<SocialTwitter>,
}
```

### 3. `src/actions/users.rs` — add profile actions

- `get_profile(user_id, conn) -> Result<Option<Profile>, DomainError>` — fetch profile, returns None if not found
- `upsert_profile(user_id, updates, conn) -> Result<Profile, DomainError>` — create or update (INSERT ... ON CONFLICT or check-then-set pattern)

### 4. `src/routes/users.rs` — add new endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/public/profiles/{user_id}` | none | Public profile view |
| GET | `/api/me/profile` | RoleUser | Own profile (returns default if none exists) |
| PATCH | `/api/me/profile` | RoleUser | Update own profile (upsert) |

Each with `#[utoipa::path(...)]` annotations, `#[protect(...)]` where needed, `#[tracing::instrument]`, and `web::block` for DB calls.

### 5. `src/lib.rs` — wire up routes

Add to the existing route scope:
```rust
.service(
    web::scope("/api/public")
        .route("/profiles/{user_id}", web::get().to(routes::users::get_public_profile)),
)
.service(
    web::scope("/api/me")
        .route("/profile", web::get().to(routes::users::get_my_profile))
        .route("/profile", web::patch().to(routes::users::update_my_profile)),
)
```

Add `Profile`, `PublicProfile`, `UpdateProfile` to the `ApiDoc` schema list.

### 6. `tests/integration/common/mod.rs` — update test `app_data`

No changes needed unless we need profile-specific test fixtures.

## Implementation Order

1. Migration SQL file
2. `schema.rs` addition
3. Newtype models in `users.rs`
4. Action functions in `actions/users.rs`
5. Route handlers in `routes/users.rs`
6. Route wiring in `lib.rs`
7. Add to OpenAPI schema list
8. Run `diesel migration run` + `cargo check`

## Notes

- `Display` impl not needed on newtypes — they're just `String` wrappers and won't be logged
- `serde(skip_serializing)` not needed on profile fields — all are serializable
- `UpdateProfile` uses `Option<Newtype>` so clients can send `null` to clear or omit to leave unchanged
- Consider auto-creating an empty profile on user signup (optional, defer for now)
