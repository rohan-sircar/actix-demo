# Phase 1 — Auth Shell Implementation Plan

## Goal

Wire the existing mock auth screens (Login, Register) to the real actix-demo API, build the auth store with JWT/session management, and create the API client. Dashboard skeleton with bottom tabs stays as-is for now.

## API Surface (actix-demo, all under `/api/v1/`)

| Method | Endpoint | Purpose | Auth |
|--------|----------|---------|------|
| `POST` | `/api/v1/registration` | Create account | No |
| `POST` | `/api/v1/login` | Login → sets X-AUTH-TOKEN cookie | No |
| `POST` | `/api/v1/logout` | Clear cookie | Yes |
| `GET` | `/api/v1/user` | Get own profile | Yes |
| `PATCH` | `/api/v1/user` | Update profile | Yes |
| `DELETE` | `/api/v1/user` | Soft delete account | Yes |
| `PUT` | `/api/v1/avatars` | Upload avatar | Yes |
| `GET` | `/api/v1/sessions` | List active sessions | Yes |
| `DELETE` | `/api/v1/sessions/{id}` | Revoke session | Yes |
| `POST` | `/api/v1/sessions/revoke-others` | Revoke all other sessions | Yes |
| `POST` | `/api/v1/email/verify` | Verify email token | No |
| `POST` | `/api/v1/password-reset/request` | Request reset link | No |
| `POST` | `/api/v1/password-reset/complete` | Complete reset | No |
| `GET` | `/api/v1/public/oauth/github` | Start GitHub OAuth | No |
| `GET` | `/api/v1/public/oauth/google` | Start Google OAuth | No |

## Tech to Add

- `axios` — HTTP client with interceptors
- `expo-secure-store` — JWT storage (encrypted)
- `react-hook-form` + `@hookform/resolvers` + `zod` — form validation

## Implementation Order

### Step 1: API Client (`lib/api.ts`)

Create an axios instance with:
- Base URL from env (default `http://192.168.1.x:8800` for local dev)
- `withCredentials: true` for cookie-based auth
- Request interceptor: nothing needed (cookies are automatic)
- Response interceptor: on 401, clear secure store and dispatch a `clearAuth` action
- Error handling: normalize errors into `{ message, field? }` shape

### Step 2: Auth Store (`app/stores/AuthStore.ts`)

Replace `UserStore.tsx` counter logic with:

```ts
interface AuthState {
  token: string | null;
  user: User | null;
  loggedIn: boolean;
  setToken: (token: string) => void;
  setUser: (user: User) => void;
  login: (email: string, password: string) => Promise<void>;
  register: (data: RegisterData) => Promise<void>;
  logout: () => Promise<void>;
  hydrate: () => Promise<void>; // read from secure store on mount
  clearAuth: () => void;
}
```

- `login()` calls `POST /api/v1/login`, reads the cookie, exchanges for a JWT via `GET /api/v1/user`, stores both in secure store
- `register()` calls `POST /api/v1/registration`
- `logout()` calls `POST /api/v1/logout`, clears secure store
- `hydrate()` runs on app mount: reads token from secure store, calls `GET /api/v1/user` to validate and fetch user data

**Important:** The backend sets an HTTP-only cookie (`X-AUTH-TOKEN`). The frontend can't read it directly. We need a server-side exchange endpoint (or use the cookie-based flow where the browser handles cookies automatically). For React Native, we'll need to add `POST /api/v1/auth/oauth/github/exchange` for OAuth, and for password auth, the cookie will be sent automatically by axios on subsequent requests.

Actually, looking at the backend more carefully: login sets an HTTP-only cookie. On the next request (GET /api/v1/user), the cookie is sent automatically. The response body contains the user data. So the flow is:
1. POST /api/v1/login → cookie is set by the server
2. GET /api/v1/user → cookie is sent automatically, returns user JSON

For the mobile app, this works if we use a WebView for the initial login (browser handles cookies), but for a native app we need the cookie to be set. Axios in React Native doesn't persist cookies by default. We have two options:
- Use `expo-web-browser` for the initial auth and extract the token
- Add a server-side exchange endpoint that returns the JWT directly

I'll plan for the exchange endpoint approach since it's cleaner for mobile.

### Step 3: Login Screen (`app/screens/LoginScreen.tsx`)

Changes needed:
- Add `react-hook-form` with zod validation for email + password
- Replace mock `useUserStore.setState({ loggedIn: true })` with real `authStore.login(email, password)`
- On success: navigate to Home > Feed (not Profile, since userId is undefined in mock store)
- Add loading state to the submit button
- Add inline error display below the form

### Step 4: Register Screen (`app/screens/RegisterScreen.tsx`)

Changes needed:
- Add `react-hook-form` with zod validation (email, password, confirmPassword)
- Wire `FormButton` to call the registration API
- On success: navigate to a "check your email" screen or directly to login
- Add loading state and error display

### Step 5: Auth Stack Wrapper (`app/components/AuthStack.tsx`)

- Add a "Forgot Password" screen route
- Wire navigation between SignIn → ForgotPassword → ResetPassword
- Add email verification result screen

### Step 6: App Mount / Layout (`app/_layout.tsx`)

- On app mount, call `authStore.hydrate()` to restore session from secure store
- Wrap the drawer in an auth guard: if not logged in, show AuthStack as the root (no drawer)
- If logged in, show the full drawer with tabs

### Step 7: Forgot Password Flow

- Create `ForgotPasswordScreen.tsx` — email input → calls `POST /api/v1/password-reset/request`
- Create `ResetPasswordScreen.tsx` — new password + confirm → calls `POST /api/v1/password-reset/complete` with token from URL params
- Create `VerifyEmailScreen.tsx` — shows success/error based on email verification result

## Files to Create

| File | Purpose |
|------|---------|
| `lib/api.ts` | Axios instance, interceptors, error normalization |
| `lib/schemas.ts` | Zod schemas for login, register, profile forms |
| `app/stores/AuthStore.ts` | JWT/session management (replaces counter-based UserStore) |
| `app/screens/ForgotPasswordScreen.tsx` | Request password reset |
| `app/screens/ResetPasswordScreen.tsx` | Complete password reset |
| `app/screens/VerifyEmailScreen.tsx` | Email verification result |

## Files to Rewrite

| File | What changes |
|------|-------------|
| `app/_layout.tsx` | Add auth guard, hydrate on mount, show AuthStack when not logged in |
| `app/screens/LoginScreen.tsx` | Wire to real API, add form validation, loading state, errors |
| `app/screens/RegisterScreen.tsx` | Wire to real API, add form validation |
| `app/components/AuthStack.tsx` | Add forgot password, reset password, verify email routes |

## Files to Modify

| File | What changes |
|------|-------------|
| `package.json` | Add `axios`, `expo-secure-store`, `react-hook-form`, `@hookform/resolvers`, `zod` |
| `app/screens/ProfileScreen.tsx` | Replace mock data with real API calls (future, Phase 2) |

## What Stays As-Is

- NativeWind component library (Button, Avatar, etc.)
- Theme system (dark mode, accent colors)
- react-query setup
- FlashList usage
- Navigation structure (drawer + tabs) — just add the auth guard wrapper
- `HomeTabs.tsx`, `MenuButton.tsx`, `DrawerContent.tsx` — these are already wired up

## Dev Setup

- API URL via `.env` or `expo-constants`: `EXPO_PUBLIC_API_URL=http://<VPS_IP>:8800`
- For local testing: run actix-demo on the VPS, use the VPS LAN IP
- `expo-secure-store` requires a dev client for physical device testing

## Acceptance Criteria

- [ ] Login with real credentials works, navigates to Home after success
- [ ] Register creates a real account
- [ ] Session persists across app restarts (secure store)
- [ ] Logout clears session and returns to login
- [ ] Form validation shows inline errors
- [ ] Loading states on all async actions
- [ ] Forgot password flow sends request
- [ ] Password reset flow completes with token
