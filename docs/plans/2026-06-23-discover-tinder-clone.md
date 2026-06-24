---
status: not-started
phase: 1
updated: 2026-06-24
---

# Implementation Plan: Discover Tab — Tinder Clone System

## Goal
Build a Tinder-style pet discovery system with Swipe deck, Grid deck, Photo Gallery, and Full Profile Page, backed by new backend API endpoints for likes and discovery.

## Context & Decisions
| Decision | Rationale | Source |
|----------|-----------|--------|
| Reuse existing `PetCard` for Grid Deck | Already styled, uses same image URL pattern, consistent UI | `ref:explore-session-1` |
| New `SwipeDeckCard` component (not reuse PetCard) | Swipe deck needs pan gestures, z-index card stack, swipe indicators — fundamentally different interaction model | — |
| Use `react-native-pager-view` for Photo Gallery | Already installed (v8.0.1), provides smooth horizontal paging | `ref:explore-session-1` |
| Use `PanGestureHandler` + `useAnimatedStyle` for Swipe Deck | reanimated 4.3.1 already configured, gives native-thread gesture performance | `ref:explore-session-1` |
| Pet View routes: `/pet-view/[pet_uuid]` and `/pet-view/profile/[pet_uuid]` | expo-router dynamic routes, same parent for shared header/navigation | `ref:explore-session-1` |
| Swipe tab inserted between Pets and Discover | Logical flow: own pets → discover/swipe → grid discover | — |
| Likes direction: enum `like` / `dislike` | Plan specifies this; swipe right=like, swipe left=dislike | Plan |
| `/pets/:uuid` private endpoint returns owner info | Full Profile Page needs owner name, avatar, pet count — requires JOIN | — |
| Message and Report endpoints are stubs | Per user instruction: implement endpoints but placeholder behavior | User |
| Keep mock data for development fallback | User requested to keep `frontend/data/pets.ts` | User |

## Phase 1: Backend — Database Migration + Models [PENDING]
- [ ] 1.1 Create Diesel migration `2026-06-24-000000_create_likes` with `likes` table schema (id, user_id, pet_owner_id, pet_id, direction, is_match, created_at) + indexes → `ref:explore-session-1`
- [ ] 1.2 Add `Like` and `NewLike` models to `src/models/likes.rs` (new file) with Diesel Queryable/Insertable derives, LikeDirection enum (like/dislike) → `ref:explore-session-1`
- [ ] 1.3 Update `src/schema.rs` via `diesel migration run` → `ref:explore-session-1`
- [ ] 1.4 Run `diesel migration run` to apply migration → `ref:explore-session-1`

## Phase 2: Backend — API Endpoints [PENDING]
- [ ] 2.1 Create `src/routes/discover.rs` with `GET /discover/next` — query: SELECT pets WHERE id NOT IN (liked/disliked pets) AND id NOT IN (pets from interacted owners), ORDER BY random(), LIMIT 1 → `ref:explore-session-1`
- [ ] 2.2 Create `GET /discover/pets` — paginated listing with optional species/gender/age_range query filters, returns Vec<PublicPet> with image URLs → `ref:explore-session-1`
- [ ] 2.3 Create `POST /likes` — accepts `{ pet_uuid, direction }`, upserts like record, checks for reciprocal like to set is_match=true → `ref:explore-session-1`
- [ ] 2.4 Extend `GET /pets/:uuid` (private) to JOIN with users table and return owner info (display_name, avatar_url, pets_owned count) alongside pet data → `ref:explore-session-1`
- [ ] 2.5 Create stubs for `POST /messages` and `POST /reports` — return 501 Not Implemented or placeholder response → `ref:explore-session-1`
- [ ] 2.6 Register all new routes in `src/lib.rs` under `/api/v1/private/discover/` and `/api/v1/private/likes/` scopes with `#[protect("RoleEnum::RoleUser")]` → `ref:explore-session-1`
- [ ] 2.7 Add utoipa path annotations for OpenAPI docs → `ref:explore-session-1`
- [ ] 2.8 Build and verify backend compiles: `cargo build` → `ref:explore-session-1`

## Phase 3: Frontend — Pet View + Full Profile [PENDING]
- [ ] 3.1 Add TypeScript types to `frontend/app/models/pets.ts`: `DiscoverPet` (PublicPet + image_url), `OwnerInfo`, `LikeRecord`, `PaginatedResponse`, `DiscoverQuery` → `ref:explore-session-1`
- [ ] 3.2 Add discovery API functions to `frontend/app/lib/api.ts`: `discoverApi.next()`, `discoverApi.list()`, `likesApi.create()`, `petApi.getWithOwner()` → `ref:explore-session-1`
- [ ] 3.3 Create `frontend/app/pet-view/[pet_uuid].tsx` — Photo Gallery screen using `react-native-pager-view` for horizontal swipe, displays pet images with info caption below (name, species, age), "Full Profile" button navigates to `/pet-view/profile/[pet_uuid]` → `ref:explore-session-1`
- [ ] 3.4 Create `frontend/app/pet-view/profile/[pet_uuid].tsx` — Full Profile Page: hero image, pet details grid (species, breed, age, weight, traits), owner mini-section (avatar, name, pets count), action button row (Match, Message, Report as stubs) → `ref:explore-session-1`
- [ ] 3.5 Wire image URLs in both screens using `${API_BASE}/api/v1/pets/images/${image_uuid}/medium` pattern → `ref:explore-session-1`

## Phase 4: Frontend — Swipe Deck [PENDING]
- [ ] 4.1 Create `frontend/app/components/SwipeDeckCard.tsx` — card component with `PanGestureHandler`, `useAnimatedStyle` for drag translation, scale, rotation; shows pet photo (full-screen), name/species/age/distance overlay, "View Photos" bottom button, swipe direction indicators (LIKE/NOPE stamps) → `ref:explore-session-1`
- [ ] 4.2 Create `frontend/app/components/SwipeDeck.tsx` — card stack container managing array of pets, z-index layering (top card scale=1, second scale=0.95, third scale=0.90), PanGestureHandler for swipe detection, onSwipeRight triggers like callback, onSwipeLeft triggers pass callback, handles "no more pets" empty state → `ref:explore-session-1`
- [ ] 4.3 Implement swipe threshold logic (e.g., 100px translation = trigger action), spring-back animation for incomplete swipes using `withSpring` from reanimated → `ref:explore-session-1`
- [ ] 4.4 Card data loading: fetch from `discoverApi.next()` on mount, refetch on like/pass → `ref:explore-session-1`

## Phase 5: Frontend — Grid Deck + Tabs [PENDING]
- [ ] 5.1 Create `frontend/app/components/FilterChips.tsx` — reusable horizontal scrollable chip row for species/gender/age filters, accent color theming consistent with existing Discover tab → `ref:explore-session-1`
- [ ] 5.2 Rewrite `frontend/app/(tabs)/discover/index.tsx` — replace mock data with real API calls to `discoverApi.list()`, add FilterChips component, wire PetCard `onPress` to navigate to `/pet-view/[pet_uuid]`, keep species filter chips, add loading/empty/error states → `ref:explore-session-1`
- [ ] 5.3 Create `frontend/app/(tabs)/swipe/index.tsx` — new Swipe tab screen wrapping `SwipeDeck` component with header "Swipe" and empty state when no more pets → `ref:explore-session-1`
- [ ] 5.4 Update `frontend/app/(tabs)/_layout.tsx` — add new "Swipe" tab between Pets and Discover, use FontAwesome `heart` icon, authenticated-only → `ref:explore-session-1`
- [ ] 5.5 Add search bar component to Discover tab (optional polish for first pass) → `ref:explore-session-1`

## Phase 6: Polish + Testing [PENDING]
- [ ] 6.1 End-to-end flow test: login → Swipe deck → like pet → Photo Gallery → Full Profile → Discover grid → filter → tap pet → Photo Gallery → Full Profile → `ref:explore-session-1`
- [ ] 6.2 Test edge cases: no more pets in swipe deck, pet without images, owner without avatar, empty discover results with filters → `ref:explore-session-1`
- [ ] 6.3 Remove loading simulation delay from Discover tab (currently 600ms simulated delay) → `ref:explore-session-1`

## Notes
- 2026-06-24: All research completed in single explore delegation. Backend follows existing patterns (actix_web, diesel, utoipa, cookie auth). Frontend follows existing patterns (expo-router, axios, reanimated, Tailwind).
- 2026-06-24: The `likes` table uses `pet_owner_id` (FK to users) rather than just tracking per-pet — this enables the match logic where two owners can match by liking each other's pets.
- 2026-06-24: The discover algorithm (Phase 2.1) is intentionally simple for v1: exclude liked/disliked pets and pets from already-interacted owners, random tiebreaker. Can be enhanced later with preferences, location, etc.
- 2026-06-24: Match detection (is_match flag) is set at like-creation time by checking for reciprocal records. A more robust approach would use a database trigger or background job, but for v1 the inline check is sufficient.
