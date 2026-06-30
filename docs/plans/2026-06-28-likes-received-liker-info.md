# Show Liker Info in Likes Received

## Goal
Display who liked your pet in the "Likes Received" tab, with a link to the liker's profile page so you can browse their pets and like one back to form a match.

## Context & Decisions
| Decision | Rationale | Source |
|----------|-----------|--------|
| Keep existing `/pet-profiles` routes unchanged | Avoid breaking changes; use new `/user-profile/[id]` for public user profiles | User preference |
| Fetch liker info from `users` + `profiles` tables via inner join | Existing pattern used by `fetch_pet_owner()` in `src/actions/pets.rs:426` | Codebase pattern |
| Use `PublicPetOwner` struct for liker data | Reuses existing serializable model with `user_uuid`, `display_name`, `avatar_url`, `pets_owned` | `src/models/pets.rs:522` |
| Show liker info in both list view and pet-view page | List view gives quick context; pet-view page gives full detail with profile link | UX research |
| `list_likes_sent` unchanged | User confirmed it correctly shows the pet they liked | User confirmation |

## Phase 1: Backend — Add Liker Info to Likes Received [PENDING]

### 1.1 Update `LikeWithPet` model to include `liker` field → `ref:plan-likes-received`
- File: `src/models/likes.rs`
- Add `pub liker: Option<PublicPetOwner>` to `LikeWithPet` struct
- `Option` because matches (from `list_matches`) don't need this field
- Reuse existing `PublicPetOwner` struct from `src/models/pets.rs`

### 1.2 Update `list_likes_received` to fetch liker info → `ref:plan-likes-received`
- File: `src/actions/likes.rs`
- For each like row, fetch the liker's user info using `like.user_id`
- Join `users` + `profiles` tables (same pattern as `fetch_pet_owner()` in `src/actions/pets.rs:432`)
- Select: `users::user_uuid`, `profiles::display_name`, `profiles::avatar_url`
- Count pets owned by liker for `pets_owned` field
- Set `liker` field on `LikeWithPet` response

### 1.3 Update `list_likes_sent` to set `liker: None`
- File: `src/actions/likes.rs`
- No changes needed for sent likes — just pass `liker: None`

### 1.4 Update `list_matches` to set `liker: None`
- File: `src/actions/likes.rs`
- Matches don't need liker info — just pass `liker: None`

### 1.5 Add endpoint to list public pets by user UUID → `ref:plan-likes-received`
- File: `src/routes/users.rs` or new `src/routes/public.rs`
- Route: `GET /api/v1/public/users/{user_uuid}/pets`
- Returns `Vec<PublicPet>` for the specified user
- Public endpoint (no auth required)
- Filter out deleted users/pets

### 1.6 Register route in `src/lib.rs`
- Add OpenAPI path registration

## Phase 2: Frontend — Update Models & API [PENDING]

### 2.1 Update `LikeWithPet` interface → `ref:plan-likes-received`
- File: `frontend/app/models/pets.ts`
- Add `liker?: { user_uuid: string; display_name: string | null; avatar_url: string | null }`

### 2.2 Add API endpoint for user's public pets → `ref:plan-likes-received`
- File: `frontend/app/lib/api.ts`
- Add `usersApi.listPublicPets(userUuid: string)` method

### 2.3 Add `UserPetProfile` interface → `ref:plan-likes-received`
- File: `frontend/app/models/pets.ts`
- Interface for the user profile page response

## Phase 3: Frontend — Update Likes UI [PENDING]

### 3.1 Show liker info in likes list item → `ref:plan-likes-received`
- File: `frontend/app/(tabs)/likes/index.tsx`
- In `LikeItem` component, add liker's display name and avatar below pet species
- Make the liker's name tappable — navigates to `/user-profile/[user_uuid]`
- Style: small text with avatar icon, similar to existing card layout

### 3.2 Show liker info on pet-view page → `ref:plan-likes-received`
- File: `frontend/app/pet-view/[pet_uuid].tsx`
- Accept `liker` prop from parent
- Display liker's name + avatar below pet details
- Tappable link to `/user-profile/[liker.user_uuid]`
- Only show for received likes (liker is present)

### 3.3 Wire up liker data in likes screen → `ref:plan-likes-received`
- File: `frontend/app/(tabs)/likes/index.tsx`
- Pass `liker` from `LikeWithPet` to `LikeItem`
- Pass `liker` to pet-view via navigation state or query param

## Phase 4: Frontend — User Profile Page [PENDING]

### 4.1 Create `/user-profile/[user_uuid]` page → `ref:plan-likes-received`
- File: `frontend/app/(tabs)/user-profile/[user_uuid].tsx`
- Fetch user's public pets via `usersApi.listPublicPets(user_uuid)`
- Display user's display_name, avatar_url, pets_owned count
- Grid of pet cards (similar to pet-profiles layout)
- Each pet card is tappable → navigates to `/pet-view/[pet_uuid]`
- Empty state: "This user has no public pets"

### 4.2 Register route in `_layout.tsx`
- File: `frontend/app/(tabs)/_layout.tsx`
- Add `Tabs.Screen` for `user-profile` if needed (probably not in bottom nav)
- Or use as a modal/screen navigated to from likes list

### 4.3 Handle loading and error states
- Loading spinner while fetching user pets
- Error state if user not found or no access

## Notes
- The existing `isOwnPet` check in pet-view prevents liking your own pet — this is correct behavior and unchanged
- The `liker` field is `Option` because only received likes have meaningful liker data
- Matches tab should NOT show liker info (both parties already matched)
- Sent likes should NOT show liker info (you know who you liked)
