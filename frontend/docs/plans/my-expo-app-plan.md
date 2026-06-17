# my-expo-app — Implementation Plan

## Current Status (as of June 12, 2026)

### Phase 1 — Auth Shell ✅ COMPLETE
- [x] Packages installed (axios, zustand, zod, react-hook-form, @hookform/resolvers, expo-secure-store, expo-web-browser, expo-router, react-query)
- [x] `lib/api.ts` — axios instance with Bearer interceptor + 401 handler + `withCredentials: true` on web
- [x] `lib/schemas.ts` — zod schemas (login, register, forgot-password, reset-password)
- [x] `stores/AuthStore.tsx` — Zustand store with platform-specific hydration (web=cookie GET /user, native=SecureStore)
- [x] LoginScreen wired to real API (web: POST /login + GET /user; native: POST /auth/exchange)
- [x] RegisterScreen wired to real API (POST /registration → POST /login or /auth/exchange → GET /user)
- [x] ForgotPasswordScreen wired to POST /password-reset/request
- [x] ResetPasswordScreen wired to POST /password-reset/complete
- [x] `app.json` — deep linking scheme `my-expo-app`, expo-secure-store plugin
- [x] CORS configured on backend (specific origin + supports_credentials)
- [x] Debug log for CORS config: `tracing::info!(cors_origins = %cors_origins, "CORS config loaded");`

### Phase 2 — Profile & Sessions ✅ COMPLETE
- [x] ProfileScreen wired to GET /user + PATCH /user/profile (via react-query)
- [x] SessionsScreen wired to GET /sessions + DELETE /sessions/{id} + POST /sessions/revoke-others
- [x] LogoutButton wired to POST /logout + clearCredentials
- [x] **SessionsScreen cleaned up:** removed raw HTML `<button>` (replaced with TouchableOpacity), removed debug console.log statements
- [x] **RegisterScreen fixed:** corrected GoogleButton import path from `'../components/GithubButton'` to `'../components/GoogleButton'`
- [x] Auth wrapper in _layout.tsx — hydrates on mount, shows loading while auth state resolves
- [x] Drawer navigation with Home/Account/Settings

### Phase 3 — OAuth ✅ COMPLETE
- [x] GithubButton wired to OAuth flow (web: redirect with cookie; native: code exchange via expo-web-browser)
- [x] GoogleButton wired to OAuth flow (web: redirect with cookie; native: code exchange via expo-web-browser)
- [x] Fixed hardcoded fallback IPs — now uses `API_BASE_URL` from env with `http://localhost:8800` fallback
- [x] Platform-specific OAuth: web uses cookie-based redirect, native uses code exchange via api.post

### Phase 4 — Pet Cards ✅ COMPLETE (basic)
- [x] Pet listing on dashboard (connect to GET /user/pets via react-query)
- [x] PetCard component with species icon, age/gender/weight badges, traits display
- [ ] Pet creation/edit flow (POST /user/pets, PATCH /user/pets/{id}) — TODO

---

## Known Issues & Artifacts

### Critical
1. ~~**SessionsScreen.tsx** — contains a raw HTML `<button>` element~~ ✅ FIXED (replaced with TouchableOpacity)
2. ~~**SessionsScreen.tsx** — extensive debug `console.log` statements~~ ✅ FIXED (all removed)
3. ~~**RegisterScreen.tsx** — wrong GoogleButton import path~~ ✅ FIXED

### Minor
4. **components/Button.tsx** — entirely commented out placeholder
5. **ProfileScreen** — "Pets: 0" stat is hardcoded (should fetch from API or count from pets list)
6. **HomeScreen** — needs pet creation flow (add pet button → create form)
7. **OAuth buttons** — web flow uses `openAuthSessionAsync` with same URL for both authorize and redirect; may need deep link scheme for production

---

## API Reference (from actix-demo)

### Auth
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/v1/login | Cookie-based login (web) |
| POST | /api/v1/auth/exchange | Token exchange (native) { username, password, device_name } → { token, user } |
| POST | /api/v1/logout | Clear session from Redis |
| GET | /api/v1/sessions | List active sessions |
| DELETE | /api/v1/sessions/{id} | Revoke specific session |
| POST | /api/v1/sessions/revoke-others | Revoke all other sessions |

### User
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/v1/registration | Create account { username, email, password } → { success, message } |
| GET | /api/v1/user | Get own profile |
| PATCH | /api/v1/user | Update profile (partial fields) |
| DELETE | /api/v1/user | Soft delete account |
| PUT | /api/v1/avatars | Upload avatar (multipart) |
| DELETE | /api/v1/avatars | Delete avatar |

### Password Reset
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/v1/password-reset/request | Request reset link { email } |
| POST | /api/v1/password-reset/complete | Set new password { token, new_password } |

### Email Verification
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/v1/email/verify | Verify email { token } |

### Pets
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/v1/user/pets | Create pet { species, breed, dob, gender, weight, traits[] } |
| GET | /api/v1/user/pets | List user's pets |
| PATCH | /api/v1/user/pets/{id} | Update pet (partial) |
| DELETE | /api/v1/user/pets/{id} | Delete pet |
| GET | /api/v1/public/pets/{id} | Public pet view |
| GET | /api/v1/public/pets/traits | Available personality traits |
| POST | /api/v1/user/pets/{id}/images | Upload pet image (multipart) |
| GET | /api/v1/user/pets/{id}/images | List pet images |
| DELETE | /api/v1/user/pets/{id}/images/{img_id} | Delete pet image |

### OAuth
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /api/v1/public/oauth/github | Start GitHub OAuth (redirect) |
| GET | /api/v1/public/oauth/github/callback | GitHub callback (redirect with cookie) |
| GET | /api/v1/public/oauth/google | Start Google OAuth (redirect) |
| GET | /api/v1/public/oauth/google/callback | Google callback (redirect with cookie) |
| POST | /api/v1/auth/oauth/github/exchange | Exchange code for token { code } → { token, user } |
| POST | /api/v1/auth/oauth/google/exchange | Exchange code for token { code } → { token, user } |

---

## Type Definitions

```typescript
export interface AuthUser {
  id: number;
  username: string;
  email: string;
}

export interface ProfileData {
  bio?: string;
  display_name?: string;
  location?: string;
  website?: string;
  social_links?: Record<string, string>;
}

export interface UserResponse {
  id: number;
  username: string;
  email: string;
  profile?: ProfileData;
}

export interface SessionInfo {
  session_id: string;
  device_id: string;
  device_name?: string;
  created_at: string;
  last_used_at: string;
  token: string;
  ttl_remaining?: number;
}

export type SessionsResponse = Record<string, SessionInfo>;

export interface Pet {
  id: string;
  species: string;
  breed?: string;
  dob?: string;
  gender?: string;
  weight?: number;
  traits?: string[];
  avatar_url?: string;
  created_at: string;
}
```

---

## Navigation Structure

```
Drawer (top-level)
├── Home (HomeTabs)
│   ├── Feed (HomeScreen)        ← mock data → pet API
│   ├── Profile (ProfileScreen)  ← real API: GET /user, PATCH /user
│   └── Sessions (SessionsScreen)← real API: sessions management
├── Account (AuthStack)
│   ├── SignIn (LoginScreen)     ← real API ✅
│   ├── Register (RegisterScreen)← real API ✅
│   ├── ForgotPassword           ← real API ✅
│   └── ResetPassword            ← real API ✅
└── Settings (ControlsScreen)    ← mock data + GET /user
```

---

## Implementation Order (Recommended)

1. ~~**Cleanup Phase 2 artifacts**~~ ✅ DONE
2. ~~**Phase 3 — OAuth**~~ ✅ DONE
3. ~~**Add `.env`**~~ ✅ DONE
4. ~~**Phase 4 — Pet cards**~~ ✅ DONE (basic listing)
5. **Pet creation flow** — add pet form (POST /user/pets) in Settings/ControlsScreen
6. **ProfileScreen** — fix "Pets: 0" stat to count from pets list
7. **OAuth production** — configure deep link scheme for OAuth callbacks
