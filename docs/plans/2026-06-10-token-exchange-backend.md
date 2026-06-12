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

### 2. `src/routes/auth.rs` — Bearer fallback in `extract()` and session endpoints

**`extract()` function (line 38):**
Critical — this function is used by `GrantsMiddleware::with_extractor` on all authenticated routes. Without updating it, Bearer-authenticated requests will pass `cookie_auth` middleware but fail at grants extraction, breaking all protected endpoints for mobile clients.

Current:
```rust
let cookie = req
    .cookie("X-AUTH-TOKEN")
    .ok_or_else(|| ErrorUnauthorized("Missing auth cookie"))?;
let token = cookie.value();
```

Change to the same cookie-first, Bearer-fallback pattern.

**`logout()` endpoint (line 247):**
Current:
```rust
let cookie = req.cookie("X-AUTH-TOKEN").ok_or_else(|| ...)?;
```

Change to the same extraction pattern (try cookie, fall back to Bearer).

**`revoke_other_sessions()` endpoint (line 337):**
Also reads `req.cookie("X-AUTH-TOKEN")` directly. Apply the same Bearer fallback pattern so mobile clients can revoke other sessions.

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

The shared session logic is already extracted into `issue_oauth_session()` (line 244), which both `github_callback()` and `google_callback()` already call. No extraction needed.

The exchange endpoints will:
1. Accept `{ code }` in JSON body
2. Call the provider's token exchange and user info endpoints directly (same as callbacks do)
3. Find or create the user
4. Call `issue_oauth_session()` to get the JWT + session
5. Return `{ token, user }` in JSON instead of redirecting with cookie

To avoid duplicating the provider interaction logic (state validation, code exchange, user info fetch), split `issue_oauth_session()` into two parts:
- A session-creation helper that takes `UserWithRoles` and returns `(token, session_info)` — used by both callbacks and exchange endpoints
- The callback functions continue to handle state/code flow → call the helper → redirect with cookie
- The exchange functions handle code flow (without state validation) → call the helper → return JSON

### 4. `src/lib.rs` — Route registration

Add new routes:
```rust
web::resource("/api/v1/auth/exchange")
    .wrap(login_limiter.clone())
    .route(web::post().to(routes::auth::exchange)),

web::scope("/api/v1/auth/oauth")
    .wrap(api_rate_limiter(&app_data.config.rate_limit.api_public))
    .route(
        "/github/exchange",
        web::post().to(routes::oauth::github_exchange),
    )
    .route(
        "/google/exchange",
        web::post().to(routes::oauth::google_exchange),
    ),
```

Note: `/api/v1/auth/exchange` uses `login_limiter` (same rate limit as login) since it is functionally equivalent. The OAuth exchange endpoints use the public API rate limiter.

Register new endpoints in `ApiDoc` struct with `#[utoipa::path]` attributes for OpenAPI documentation.

## Shared Helper Functions

### `create_session()` — Unified session creation

Takes: `UserWithRoles`, `device_name: Option<String>`, `&AppData`
Returns: `(jwt_token: String, session_info: SessionInfo)`

Does:
1. Generate session_id + device_id
2. Build `VerifiedAuthDetails` claims
3. Sign JWT with 30-day TTL
4. Create session in Redis
5. Return token + session info

This single helper covers both password login (`login()` + `exchange()`) and OAuth flows (`issue_oauth_session()` passes `device_name: None`). Consolidates the two proposed helpers (`create_session` and `issue_session`) into one, since they differ only by `device_name`.

## Summary of Changes

| File | Change | Lines |
|------|--------|-------|
| `src/utils/cookie_auth.rs` | Add Bearer header fallback in 3 places | ~15 |
| `src/routes/auth.rs` | Update `extract()`, `logout()`, `revoke_other_sessions()`; extract `create_session()`, add `exchange()` endpoint | ~140 |
| `src/routes/oauth.rs` | Split `issue_oauth_session()` into session helper + response builder; add `github_exchange()` + `google_exchange()` | ~110 |
| `src/lib.rs` | Register new routes with rate limiters; add to OpenAPI doc | ~15 |
| **Total** | | **~280 lines** |

## Testing

1. `curl -X POST http://localhost:7800/api/v1/auth/exchange -H "Content-Type: application/json" -d '{"username":"test","password":"test"}'` → returns `{ token, user }`
2. Use returned token: `curl http://localhost:7800/api/v1/user -H "Authorization: Bearer <token>"` → returns user data
3. OAuth exchange: `curl -X POST http://localhost:7800/api/v1/auth/oauth/github/exchange -H "Content-Type: application/json" -d '{"code":"xxx"}'` → returns `{ token, user }`
4. Verify cookie auth still works: login with browser → cookie is set → subsequent requests work
5. Verify `revoke_other_sessions` works with Bearer: `curl -X POST http://localhost:7800/api/v1/sessions/revoke-others -H "Authorization: Bearer <token>"`
6. Verify logout works with Bearer: `curl -X POST http://localhost:7800/api/v1/logout -H "Authorization: Bearer <token>"`

## Notes

- Cookie auth is the **default** — cookie is checked first, Bearer is the fallback. This means existing browser clients are unaffected.
- The `logout()` and `revoke_other_sessions()` endpoints need the Bearer fallback because the mobile app can't read the HttpOnly cookie to clear it.
- The `extract()` function update is critical — without it, `GrantsMiddleware` will reject all Bearer-authenticated requests on protected routes.
- OAuth exchange endpoints don't need state validation (the code is already validated by GitHub/Google when the mobile app receives it via custom scheme callback).
- Consider adding a `device_name` field to the exchange request body (defaults to "Mobile" if not provided).
- The `/api/v1/auth/exchange` endpoint uses the login rate limiter to prevent brute-force attacks.
