# Fix GrantsMiddleware to Actually Enforce Roles

## Problem

`GrantsMiddleware::with_extractor(routes::auth::extract)` wraps the `/api` scope in `lib.rs:228`, and the `extract()` function correctly reads the JWT and returns the user's roles as a `HashSet<RoleEnum>`. However, no route handler has any `#[protect(...)]` attribute, so the middleware extracts roles but nothing ever checks them. Every authenticated user gets full access to every endpoint regardless of their role claims.

## Current State

- `RoleEnum` is defined in `src/models/roles.rs` with variants: `RoleSuperUser`, `RoleAdmin`, `RoleUser`
- JWT claims include the user's roles (serialized as `Vec<RoleEnum>`)
- `extract()` in `src/routes/auth.rs` returns `HashSet<RoleEnum>` from JWT claims
- `GrantsMiddleware` is configured but no routes use `#[protect(...)]` attributes
- `actix-web-grants` crate is already a dependency

## Proposed Route Split

### Admin-only routes (require RoleAdmin or RoleSuperUser)
- `POST /api/cmd` — execute background job
- `GET /api/cmd/{job_id}` — get job details
- `DELETE /api/cmd/{job_id}` — abort job
- `GET /api/sessions` — list all sessions
- `DELETE /api/sessions/{session_id}` — revoke specific session
- `POST /api/sessions/revoke-others` — revoke other sessions
- `GET /api/public/users` — list all users (public scope)
- `GET /api/public/users/search` — search users (public scope)

### Self-service routes (any authenticated user)
- `PUT /api/avatars` — upload avatar
- `DELETE /api/avatars` — delete avatar
- `GET /api/users/me` — get my profile
- `PATCH /api/users/me` — update my profile
- `POST /api/users/me/delete` — delete my account

## Implementation Approach

1. Add `#[protect(RoleEnum::RoleAdmin)]` (or `RoleSuperUser`) attributes on admin-only route handlers using `actix-web-grants` syntax
2. Optionally add middleware-level fallback to return 403 Forbidden for unauthorized access (default behavior of grants middleware)
3. Update any routes that need role-based access control

## Files to Touch

- `src/routes/auth.rs` — add `#[protect(...)]` attributes on session management handlers
- `src/routes/command.rs` — add `#[protect(...)]` on job execution handlers
- `src/routes/users.rs` — add `#[protect(...)]` on user management handlers (list/search)
- `src/lib.rs` — verify middleware stack is correct, may remove the `Condition::new(true, ...)` wrapper if it's no longer needed

## Verification

- Test that an admin user can access all routes
- Test that a regular user gets 403 on admin-only routes
- Test that a regular user can still access self-service routes
- Test that unauthenticated users get 401 on all protected routes
