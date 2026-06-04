# Admin API Structure

## Goal

Split the monolithic `/api/users` scope into:
- **Self-service:** `/api/user` (singular) — users manage themselves
- **Admin:** `/api/admin/users` (plural, role-guarded) — admin-only user management

## Current State

```
/api/users (authenticated scope)
├── GET    /me                  — self-service profile view
├── PATCH  /                    — self-service profile update
├── POST   /me/delete           — self-service account deletion
├── GET    /{user_id}           — admin: get specific user ⚠️ no guard
├── GET    /                    — admin: list all users ⚠️ no guard
└── GET    /search              — admin: search users ⚠️ no guard
```

## Target State

```
/api/user (self-service, authenticated)
├── GET    /me                  — my profile
├── PATCH  /                    — update my profile
└── POST   /me/delete           — delete my account

/api/admin/users (admin, role-guarded)
├── GET    /                    — list all users          #[protect("RoleEnum::RoleAdmin")]
├── GET    /search              — search users            #[protect("RoleEnum::RoleAdmin")]
└── GET    /{user_id}           — get specific user       #[protect("RoleEnum::RoleAdmin")]

/api/public/users (public, rate-limited)
├── GET    /                    — list all users          (no auth)
├── GET    /search              — search users            (no auth)
└── GET    /{user_id}           — get specific user       (no auth)
```

## Changes

### 1. `src/lib.rs` — route restructuring

**a) Rename self-service scope from `/api/users` to `/api/user`:**
```rust
// Before
.service(
    web::scope("/users")
        .route("/me", ...)
        .route("", web::patch().to(...))
        .route("/me/delete", ...)
        .route("/{user_id}", ...)      // ← remove these 3
        .route("", web::get().to(...))
        .route("/search", ...)
)

// After
.service(
    web::scope("/user")              // ← singular
        .route("/me", ...)
        .route("", web::patch().to(...))
        .route("/me/delete", ...)
)
```

**b) Add admin scope with role guard:**
```rust
.service(
    web::scope("/admin")
        .wrap(GrantsMiddleware::with_extractor(routes::auth::extract))
        .service(
            web::scope("/users")
                .route("", web::get().to(routes::users::get_users))
                .route("/search", web::get().to(routes::users::search_users))
                .route("/{user_id}", web::get().to(routes::users::get_user))
        )
)
```

### 2. `src/routes/users.rs` — add role guards

Add `#[protect("RoleEnum::RoleAdmin", ty = RoleEnum)]` to:

```rust
#[protect("RoleEnum::RoleAdmin", ty = RoleEnum)]
pub async fn get_user(...) { ... }

#[protect("RoleEnum::RoleAdmin", ty = RoleEnum)]
pub async fn get_users(...) { ... }

#[protect("RoleEnum::RoleAdmin", ty = Role_enum)]
pub async fn search_users(...) { ... }
```

### 3. `tests/integration/role_enforcement.rs` — add admin user endpoint tests

Add a new test module `admin_user_route_access`:

```rust
mod admin_user_route_access {
    #[actix_rt::test]
    async fn should_return_403_for_non_admin_on_get_users() {
        // Register RoleUser, login, GET /api/admin/users → 403
    }

    #[actix_rt::test]
    async fn should_return_403_for_non_admin_on_search_users() {
        // Register RoleUser, login, GET /api/admin/users/search?q=test → 403
    }

    #[actix_rt::test]
    async fn should_return_403_for_non_admin_on_get_user() {
        // Register RoleUser, login, GET /api/admin/users/{id} → 403
    }

    #[actix_rt::test]
    async fn should_allow_admin_on_admin_user_routes() {
        // Login as RoleAdmin, GET /api/admin/users → NOT 403
    }
}
```

### 4. `tests/integration/role_enforcement.rs` — update self-service test

Update `self_service_route_access` to use the new `/api/user/me` path:

```rust
// Before
.get("/api/users/me")

// After
.get("/api/user/me")
```

### 5. `tests/integration/common/mod.rs` — update any hardcoded paths

Search for `/api/users` references in test helpers and update:
- `get_users`, `search_users`, `get_user` test helpers → `/api/admin/users`
- Self-service helpers → `/api/user/me`, `/api/user`, `/api/user/me/delete`

## Verification

1. `cargo test` — all tests pass
2. Non-admin user hits `/api/admin/users` → 403
3. Admin user hits `/api/admin/users` → 200 (or handler-specific response, NOT 403)
4. Any authenticated user hits `/api/user/me` → 200
5. `curl` smoke test against running instance
