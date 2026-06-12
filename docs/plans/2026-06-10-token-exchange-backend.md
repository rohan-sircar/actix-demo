# Backend Plan — Token Exchange Endpoints

## Goal

Add token-based auth alongside existing cookie-based auth so the React Native frontend can authenticate via `Authorization: Bearer` header. Cookie auth stays intact for web/browser clients.

## Files to Modify

### 1. `src/utils/cookie_auth.rs` — Auth extraction (3 places)

**`cookie_auth` middleware (line 72):**
```rust
// Current: only reads cookie
let cookie = req.cookie("X-AUTH-TOKEN");
let token = match cookie { ... };

// Change to: try cookie first, fall back to Bearer header
let token = req
    .cookie("X-AUTH-TOKEN")
    .map(|c| c.value().to_string())
    .or_else(|| {
        req.headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string())
    })
    .ok_or_else(|| ErrorUnauthorized("Missing auth token"))?;
```

**`CookieAuth` extractor (line 24):**
Same change — try cookie, fall back to `Authorization: Bearer`.

**`extract_auth_token` function (line 143):**
Same change — try cookie, fall back to `Authorization: Bearer`.

This is a single pattern applied in 3 places. ~5 lines of code each.

### 2. `src/routes/auth.rs` — New exchange endpoints

**`POST /api/v1/auth/exchange`** — Password-based token exchange

```rust
pub async fn exchange(
    login_request: web::Json<UserLogin>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    // Same validation logic as login() — verify credentials
    // Same session creation — create JWT + store in Redis
    // Return JSON instead of setting cookie:
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user": { "id": ..., "username": ..., "email": ... }
    })))
}
```

The body is ~95% duplicate of `login()`. Extract the shared logic (credential verification + session creation) into a private helper:

```rust
// Shared logic extracted from login()
async fn create_session(
    user: UserWithRoles,
    device_name: Option<String>,
    app_data: &AppData,
) -> Result<(String, SessionInfo), DomainError> { ... }
```

Then `login()` calls `create_session()` + sets cookie, `exchange()` calls `create_session()` + returns JSON.

### 3. `src/routes/oauth.rs` — New OAuth exchange endpoints

**`POST /api/v1/auth/oauth/github/exchange`** and **`POST /api/v1/auth/oauth/google/exchange`**

Extract the shared logic from `github_callback()` / `google_callback()` into a private helper:

```rust
async fn issue_session(
    user: UserWithRoles,
    app_data: &AppData,
) -> Result<(String, serde_json::Value), DomainError> { ... }
```

The callback endpoints call `issue_session()` + redirect with cookie.
The exchange endpoints call `issue_session()` + return `{ token, user }` in JSON.

### 4. `src/routes/auth.rs` — Logout endpoint

Current logout reads the cookie directly (line 247):
```rust
let cookie = req.cookie("X-AUTH-TOKEN").ok_or_else(|| ...)?;
```

Change to use the same extraction pattern as the middleware (try cookie, fall back to Bearer).

### 5. `src/lib.rs` — Route registration

Add new routes:
```rust
web::resource("/api/v1/auth/exchange")
    .route(web::post().to(routes::auth::exchange)),

web::scope("/api/v1/auth/oauth")
    .route(
        "/github/exchange",
        web::post().to(routes::oauth::github_exchange),
    )
    .route(
        "/google/exchange",
        web::post().to(routes::oauth::google_exchange),
    )
```

## Shared Helper Functions

### `create_session()` — Extracted from login()

Takes: `UserWithRoles`, `device_name: Option<String>`, `&AppData`
Returns: `(jwt_token: String, session_info: SessionInfo)`

Does:
1. Generate session_id + device_id
2. Build `VerifiedAuthDetails` claims
3. Sign JWT with 30-day TTL
4. Create session in Redis
5. Return token + session info

### `issue_session()` — Extracted from OAuth callbacks

Takes: `UserWithRoles`, `&AppData`
Returns: `(jwt_token: String, session_info: SessionInfo)`

Same as `create_session()` but for OAuth users (device_name is None).

## Summary of Changes

| File | Change | Lines |
|------|--------|-------|
| `src/utils/cookie_auth.rs` | Add Bearer header fallback in 3 places | ~15 |
| `src/routes/auth.rs` | Extract `create_session()`, add `exchange()` endpoint, update `logout()` | ~120 |
| `src/routes/oauth.rs` | Extract `issue_session()`, add `github_exchange()` + `google_exchange()`, update callbacks to use helper | ~100 |
| `src/lib.rs` | Register new routes | ~10 |
| **Total** | | **~245 lines** |

## Testing

1. `curl -X POST http://localhost:7800/api/v1/auth/exchange -H "Content-Type: application/json" -d '{"username":"test","password":"test"}'` → returns `{ token, user }`
2. Use returned token: `curl http://localhost:7800/api/v1/user -H "Authorization: Bearer <token>"` → returns user data
3. OAuth exchange: `curl -X POST http://localhost:7800/api/v1/auth/oauth/github/exchange -H "Content-Type: application/json" -d '{"code":"xxx"}'` → returns `{ token, user }`
4. Verify cookie auth still works: login with browser → cookie is set → subsequent requests work

## Notes

- Cookie auth is the **default** — cookie is checked first, Bearer is the fallback. This means existing browser clients are unaffected.
- The `logout()` endpoint needs the Bearer fallback because the mobile app can't read the HttpOnly cookie to clear it.
- OAuth exchange endpoints don't need state validation (the code is already validated by GitHub/Google when the mobile app receives it via custom scheme callback).
- Consider adding a `device_name` field to the exchange request body (defaults to "Mobile" if not provided).
