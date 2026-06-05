# OpenAPI/Swagger API Documentation

**Goal:** Add OpenAPI spec generation and a Redoc UI at `/api/docs` so the API is self-documenting.

## Approach

Use `utoipa` + `utoipa-redoc`:
- **`utoipa`** generates OpenAPI spec from Rust code annotations at compile time
- **`utoipa-redoc`** serves a Redoc UI from the generated spec
- Runtime spec generation (simpler, no build-time complexity)
- Redoc chosen over Swagger UI — cleaner, better for API docs, less config needed

## Dependencies to Add

```toml
utoipa = { version = "5", features = ["actix_extras", "chrono", "uuid"] }
utoipa-redoc = "4"
```

## Implementation Steps

### 1. Annotate request/response structs with `#[derive(ToSchema)]`

Add `ToSchema` derive to all public DTOs that appear in API responses or request bodies:

**Users module (`src/models/users.rs`):**
- `NewUser` — registration request body
- `UserLogin` — login request body
- `UpdateUserProfile` — profile update request body
- `User` — user response
- `UserWithRoles` — admin user response
- `OAuthProvider` — OAuth provider enum

**Auth module (`src/routes/auth.rs`):**
- `VerifyEmailRequest` — email verify request
- `PasswordResetRequest` — password reset request
- `PasswordResetCompleteRequest` — password reset complete request

**Command module (`src/routes/command.rs`):**
- `RunCommandRequest` — job execution request

**OAuth module (`src/services/oauth/models.rs`):**
- `GitHubOAuthUser`, `GitHubEmail`, `GitHubTokenResponse`
- `GoogleOAuthUser`, `GoogleTokenResponse`

**Misc module (`src/models/misc.rs`):**
- `Job`, `NewJob`, `JobCount` — job responses

**Session module (`src/models/session.rs`):**
- `SessionInfo` — session listing response

**Error handling:**
- `Error` (from `src/models/misc.rs`) — standard error response schema

### 2. Annotate handler functions with `#[utoipa::path(...)]`

Add path annotations to all ~25 handler functions. Pattern:

```rust
#[utoipa::path(
    post,
    path = "/api/login",
    tag = "auth",
    request_body = UserLogin,
    responses(
        (status = 200, description = "Login successful", body = UserWithRoles),
        (status = 401, description = "Invalid credentials", body = Error),
    ),
)]
pub async fn login(...) -> Result<impl Responder, DomainError> { ... }
```

**Route groups (tags):**
- `auth` — login, logout, email verify, password reset, sessions, account deletion
- `users` — registration, profile management, avatars, admin endpoints
- `oauth` — GitHub/Google login + callback
- `command` — job execution, job metrics
- `public` — health check, build info, public user lookup

### 3. Generate OpenAPI spec

Create a central OpenAPI spec builder, likely in `src/lib.rs` or a new `src/docs.rs`:

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::auth::login,
        routes::auth::logout,
        // ... all handlers
    ),
    components(
        schemas(
            NewUser, UserLogin, UpdateUserProfile, User, UserWithRoles,
            VerifyEmailRequest, PasswordResetRequest, PasswordResetCompleteRequest,
            RunCommandRequest,
            GitHubOAuthUser, GoogleOAuthUser,
            Job, NewJob, JobCount,
            SessionInfo,
            Error,
        ),
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "oauth", description = "OAuth 2.0 endpoints"),
        (name = "command", description = "Background job execution"),
        (name = "public", description = "Public endpoints"),
    ),
)]
pub struct ApiDoc;
```

### 4. Serve spec + UI at `/api/docs`

In `configure_app`, add the Redoc service:

```rust
use utoipa_redoc::{Redoc, Servable};

// In configure_app:
cfg.add_service(
    Redoc::with_spec(ApiDoc::openapi())
        .route("/api/docs", web::get().to(|| async { /* serve */ })),
);
```

### 5. Handle `DomainError` → OpenAPI responses

Map `DomainError` variants to HTTP status codes in the response annotations:
- `new_auth_error` → 401
- `new_bad_input_error` → 400
- `new_not_found_error` → 404
- `new_internal_error` → 500
- `new_forbidden_error` → 403

Consider creating a unified `Error` response schema that documents all possible error codes.

### 6. Optional: Compile-time spec generation

If startup performance matters, use `utoipa/gen` to generate the spec at compile time:
```toml
utoipa = { version = "5", features = ["actix_extras", "chrono", "uuid", "gen"] }
```

This generates `openapi.json` as a build artifact. Runtime serving just reads the pre-generated file.

## Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` | Add `utoipa`, `utoipa-redoc` |
| `src/models/users.rs` | Add `ToSchema` derives to DTOs |
| `src/models/misc.rs` | Add `ToSchema` to `Error`, `Job`, `NewJob`, `JobCount` |
| `src/models/session.rs` | Add `ToSchema` to `SessionInfo` |
| `src/routes/auth.rs` | Add `ToSchema` + `#[utoipa::path]` to handlers |
| `src/routes/users.rs` | Add `ToSchema` + `#[utoipa::path]` to handlers |
| `src/routes/oauth.rs` | Add `ToSchema` + `#[utoipa::path]` to handlers |
| `src/routes/command.rs` | Add `ToSchema` + `#[utoipa::path]` to handlers |
| `src/routes/healthcheck.rs` | Add `#[utoipa::path]` |
| `src/routes/misc.rs` | Add `#[utoipa::path]` |
| `src/services/oauth/models.rs` | Add `ToSchema` to DTOs |
| `src/lib.rs` | Add `ApiDoc` struct + Redoc service registration |

## Testing

- No new tests needed — the spec is generated from code annotations
- Manual testing: navigate to `/api/docs`, verify all endpoints appear, try "Try it out" on a few endpoints
- CI check: ensure `cargo build` succeeds with the new derive macros

## Out of Scope

- OpenAPI validation in CI (can add later)
- API versioning in the spec (noted as a P2 item in project analysis)
- Interactive examples / sample requests (can be added later)
- Authentication flow documentation for OAuth PKCE (documented in the spec but no special handling needed)
