---
status: in-progress
phase: 4G
updated: 2026-06-25
---

# Implementation Plan: Swipe Tab - Profile Swipe System (Native)

## Goal
Create a new "Swipe" tab that brings up a Tinder-style swipe deck for browsing pet profiles. User can drag cards left/right or use buttons to like/dislike, which loads the next pet. Real API integration with `discoverApi.next()` and `likesApi.create()`.

## Context & Decisions
| Decision | Rationale | Source |
|----------|-----------|--------|
| Swipe tab between Pets and Discover | Logical flow: own pets → swipe → grid discover | Plan Phase 5 (original) |
| Use mock data from `frontend/data/pets.ts` for initial UI dev | No real pet profiles in system yet; existing mock data has 10 pets | User instruction |
| Real API integration: `discoverApi.next()` + `likesApi.create()` | Backend endpoints exist and are tested (Phase 2) | Plan Phase 2 |
| Adapt NativeGallery layout as swipe card base | Already has Tinder-style layout (photo overlay, pills, action buttons); swipe deck adds pan gesture on top | User reference |
| `Gesture.Pan()` builder + `GestureDetector` for drag | reanimated 4.3.1 + gesture handler ~2.31.1; v2 API requires builder pattern (not `usePanGesture` hook which is v3-only) | Runtime error fix |
| Card stack: 3 cards visible (top scale=1, second scale=0.95, third scale=0.90) | Visual depth cue; standard Tinder pattern | Plan Phase 4 (original) |
| Swipe threshold: 100px translation | Standard Tinder-like threshold; not too sensitive, not too strict | Plan Phase 4 (original) |
| `withSpring` for incomplete swipe snap-back | reanimated provides smooth spring physics | Plan Phase 4 (original) |
| Mock data uses existing `MockPet` interface | No need to create new types; adapt existing data shape | — |
| Image URLs use placeholder (colored gradient backgrounds) | No real pet images in mock data; avoid broken image links | — |

## Execution Strategy
Sequential phases only. Each phase has a verifiable deliverable at the end.

---

## Phase 4A: Swipe Screen Shell + Tab Registration [DONE]
- [x] 4A.1 Create `frontend/app/(tabs)/swipe/index.tsx` — Swipe tab screen shell with "Swipe" title, placeholder content area, authentication guard (redirect to login if not authenticated)
- [x] 4A.2 Update `frontend/app/(tabs)/_layout.tsx` — add new "Swipe" tab between Pets and Discover, use FontAwesome `heart` icon, authenticated-only, title "Swipe"

**Phase 4A deliverable:** Tapping Swipe tab shows a screen with "Swipe" header and placeholder content. Tab appears in bottom bar between Pets and Discover with heart icon. Not logged in → redirects to login.

---

## Phase 4B: SwipeDeckCard Component [DONE]
- [x] 4B.1 Create `frontend/app/components/SwipeDeckCard.tsx` — card component wrapping a pet's full-screen photo with overlaid info:
  - Top bar: filter icon (dummy), category tabs (dummy), lightning bolt (dummy) — same as NativeGallery top bar
  - Photo area: full-card background image (gradient placeholder for mock data), left/right half tap zones for prev/next card navigation
  - Page dots at top center (for multi-image pets)
  - Info overlay at bottom: trait pills (species/breed, gender, age), name + age row with arrow button, action button row (undo dummy, X/dislike, star dummy, heart/like, message→profile)
  - Swipe indicators: "LIKE" stamp (green, rotated right) on right side, "NOPE" stamp (red, rotated left) on left side — appear during drag past threshold zone
- [x] 4B.2 Wire `Gesture.Pan()` + `GestureDetector` to card: track `translationX` and `translationY`, apply `useAnimatedStyle` for:
  - Horizontal translation (follows finger)
  - Rotation: interpolate from -0.15 to 0.15 radians based on translationX
  - Scale: slight scale down as card moves (optional polish)
  - Opacity of LIKE/NOPE stamps: appear when translationX exceeds threshold zone (75px)
- [x] 4B.3 Implement release behavior:
  - If `translationX > 100`: animate card flying off-screen right with `withSpring`, trigger `onLike` callback
  - If `translationX < -100`: animate card flying off-screen left with `withSpring`, trigger `onDislike` callback
  - If within threshold: spring back to center with `withSpring`
- [x] 4B.4 Accept props: `pet`, `colors`, `accentSet`, `isDarkColorScheme`, `onLike`, `onDislike`, `onFullProfile`, `isTopCard` (boolean — only top card is interactive)

**Note:** Initially used `PanGestureHandler` (old API) which caused runtime error "Failed to obtain view for PanGestureHandler". Fixed by switching to `Gesture.Pan()` builder + `GestureDetector` (v2 API).

---

## Phase 4C: SwipeDeck Container [DONE]
- [x] 4C.1 Create `frontend/app/components/SwipeDeck.tsx` — card stack container:
  - Manages array of pets (first 3 for stack depth)
  - Z-index layering: top card `zIndex: 3`, second `zIndex: 2`, third `zIndex: 1`
  - Scale + translate for stacked cards: second card scaled to 0.95 and shifted up 8px, third card scaled to 0.90 and shifted up 16px
  - Only top card (`isTopCard=true`) receives GestureDetector; lower cards are static
  - When top card swipes away: pop from array, load new pet (or show empty state if exhausted)
- [x] 4C.2 Implement "no more pets" empty state: centered text "No more pets to discover" with refresh button
- [x] 4C.3 Expose `onLike`/`onDislike` callbacks that:
  - Trigger next card load
  - Animate card out, then slide remaining cards up

---

## Phase 4D: Wire Screen + Navigation [DONE]
- [x] 4D.1 Update `frontend/app/(tabs)/swipe/index.tsx` — render `SwipeDeck` with mock data, pass color/accent props from `useColorScheme`/`useAccentColor`
- [x] 4D.2 Wire action buttons:
  - Heart (like) → call `onLike`, animate card out, load next pet
  - X (dislike) → call `onDislike`, animate card out, load next pet
  - Arrow-up (full profile) → navigate to `/pet-view/profile/[pet_uuid]` (for mock pets, use mock UUID)
  - Star and undo → dummy buttons (no-op for now)
- [x] 4D.3 Handle mock data exhaustion: when all pets have been swiped, show empty state with "Start Over" button that resets the deck

---

## Phase 4E: Polish + Visual Refinement [DONE]
- [x] 4E.1 Add subtle shadow to top card (elevation 8 on native) for depth against stacked cards — deeper shadow on top card, lighter on stacked cards
- [x] 4E.2 Smooth card entry animation: when new card enters from bottom, use `withSpring` slide-up with staggered delay per stack index
- [x] 4E.3 Ensure safe area padding doesn't interfere with swipe gesture area — `useSafeAreaInsets` used for top/bottom padding in SwipeDeck container
- [x] 4E.4 Verify gesture doesn't conflict with scroll or other touch areas — `GestureDetector` only on the top card
- [x] 4E.5 Verify both light and dark mode rendering (photo overlay contrast, stamp colors, pill backgrounds)

**Phase 4E deliverable:** Swipe tab loads a stack of 3 pet cards. Drag top card left/right — it follows finger with rotation. Release past threshold: card flies off-screen with animation, next card loads with slide-up. Release before threshold: card springs back. Action buttons work (like/dislike advance deck, full profile navigates, others are dummies). Empty state shows when deck exhausted. Works in both light and dark mode.

---

## Phase 4F: Real API Integration [PENDING]
- [ ] 4F.1 Update `frontend/app/(tabs)/swipe/index.tsx` — replace mock data with `discoverApi.next()` calls on mount and after each swipe action
- [ ] 4F.2 Update `SwipeDeck` container to accept `PublicPet` type (from API) instead of `MockPet`:
  - Map API response fields to card props: `pet.name`, `pet.species`, `pet.breed`, `pet.gender`, `pet.date_of_birth` → `getAgeFromDob()`, `pet.traits[]`, `pet.images[0]` for photo
  - Handle loading state: show `ActivityIndicator` while fetching first pet
  - Handle empty response from `GET /discover/next`: show "no more pets" empty state
- [ ] 4F.3 Wire `onLike`/`onDislike` to call `likesApi.create({ petId, direction: 'like' | 'dislike' })`:
  - Optimistic UI: animate card out immediately, then fire API call
  - On API success: fetch next pet via `discoverApi.next()`
  - On API failure: show error toast, spring card back
- [ ] 4F.4 Handle real pet images in `SwipeDeckCard`:
  - Replace gradient placeholder with `Image` component using `pet.images[0].uuid`
  - Fallback to gradient background if image fails to load
  - Use `getImageUrl(imageUuid, 'medium')` for optimized image size
- [ ] 4F.5 Implement "pull to refresh" / "load more" pattern:
  - When deck is exhausted (API returns empty), show empty state with "Check Back Later" message
  - Optional: add pull-to-refresh to re-fetch from `GET /discover/next`

**Phase 4F deliverable:** Swipe tab loads real pet data from `GET /discover/next`. Swiping left/right calls `POST /likes` with appropriate direction. Next pet loads automatically after each swipe. Real pet images display on cards. Loading states and error handling work correctly.

---

## Phase 4G: Web Swipe Deck [IN PROGRESS]
- [x] 4G.1 Create `frontend/app/components/SwipeDeckWeb.tsx` — web-specific swipe deck using CSS transforms + mouse events
  - Card stack container with state management, empty state, reset button
  - Max-width constraint (430px) centered on desktop
- [x] 4G.2 Create `frontend/app/components/SwipeDeckCardWeb.tsx` — web-specific card component:
  - Same layout as native `SwipeDeckCard` using HTML elements
  - `onMouseDown`/`onMouseMove`/`onMouseUp` drag interaction
  - CSS transitions for fly-out animation (`transition: transform 300ms ease-out`)
  - CSS-based LIKE/NOPE stamp opacity transitions
  - Keyboard support: left/right arrow keys for dislike/like
  - Gradient backgrounds with pet emoji placeholders
- [x] 4G.3 Update `frontend/app/(tabs)/swipe/index.tsx` — platform-specific rendering:
  - Detect `Platform.OS === 'web'` and render `SwipeDeckWeb` vs native `SwipeDeck`
  - Share same mock data, color/accent props
- [ ] **4G.4 Verify responsive layout + test on web** ← CURRENT
  - Card width constrained on desktop (max ~400px, centered)
  - Full-width on mobile viewport
  - Action buttons scale appropriately

**Phase 4G deliverable:** Swipe tab works on web with mouse drag, keyboard shortcuts (left/right arrows), and CSS animations. Visual layout matches native version. Responsive on desktop and mobile viewports.

---

## Phase 4H: Seed Data [COMPLETE]
- [x] 4H.1 Create `examples/seed.rs` — CLI example that connects to PostgreSQL via `ACTIX_DEMO_DATABASE_URL` and inserts seed data using Diesel
- [x] 4H.2 **Users:** Create accounts with bcrypt-hashed passwords:
  - `swiper` (password: `password123`) — user who will be swiping/discovering
  - `petowner1` (password: `password123`) — owns 6+ pets
  - `petowner2` (password: `password123`) — owns 6+ pets
  - All assigned `role_user` role
  - Idempotent: skip if username already exists
- [x] 4H.3 **Profiles:** Create profiles for all users with display_name, bio, location
- [x] 4H.4 **Pets:** Create ~12 pets owned by petowner1/petowner2:
  - Mix of species: dogs (Golden Retriever, French Bulldog, Corgi, Labrador), cats (Siamese, Maine Coon, Orange Tabby), birds
  - Various genders, ages (DOB), breeds, weights
  - Assign personality traits from existing `personality_traits` table (playful, calm, energetic, etc.)
  - Pet images: Upload from `examples/seed_images/{pet_name}/` to MinIO, create `pet_images` records
- [x] 4H.5 **Verify:** Run seed, then test `GET /discover/next` returns a pet for the `swiper` user

**Phase 4H deliverable:** `cargo run --bin seed` creates users, profiles, and pets. After seeding, logging in as `swiper` and calling `GET /discover/next` returns a random pet from petowner1/petowner2 that swiper hasn't interacted with.

---

## Notes
- Mock images: since mock data has no real image URLs, use `View` with gradient backgrounds (e.g., warm tones for dogs, cool tones for cats) or generate placeholder SVGs. This avoids broken image links during development. Phase 4F replaces this with real pet images.
- The `NativeGallery` component's layout structure (top bar → photo area with tap zones → dots → info overlay) should be reused as-is inside `SwipeDeckCard`. The only addition is the `GestureDetector` wrapper and swipe animation logic.
- Action buttons in `NativeGallery` already wire to `onLike`/`onDislike`/`onFullProfile` — these callbacks propagate from `SwipeDeckCard` → `SwipeDeck` → screen handler.
- Keep the existing `NativeGallery.tsx` untouched — it serves the photo gallery use case. `SwipeDeckCard.tsx` should be a new component that adapts the same visual layout for the swipe context.
- **Gesture API version**: `react-native-gesture-handler@~2.31.1` uses `Gesture.Pan()` builder + `GestureDetector` (not `usePanGesture` hook which is v3-only, not `PanGestureHandler` which fails with functional components).
- **Web gesture handling**: `react-native-gesture-handler` is not available on web. Use `onMouseDown`/`onMouseMove`/`onMouseUp` + CSS transforms for drag interaction. CSS transitions handle fly-out animations.
- **Platform-specific split**: Same pattern as Photo Gallery — base `[pet_uuid].tsx` handles data fetching and renders `SwipeDeckWeb` or native `SwipeDeck` based on `Platform.OS`.
- **API endpoints available**: `GET /api/v1/private/discover/next` (next pet), `POST /api/v1/private/likes` (create like/dislike), `GET /api/v1/private/discover/pets` (list pets)
