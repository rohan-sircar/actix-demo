# Explicit Reciprocal Pet Selection for Matches

## Goal

Remove automatic match detection from `create_like`. Replace with an explicit selection flow: when User A clicks "Like" on User B's pet AND User B has already liked one of User A's pets, show a popup asking User A to choose which of their own pets to match with. The selected reciprocal pet UUID is passed to the backend as part of the like creation request.

**Current behavior:** `create_like()` auto-detects reciprocal likes and creates matches automatically. This led to duplicate matches when one user liked multiple pets of another — the same reciprocal like was paired with each new like, creating multiple match rows for the same pair.

**Desired behavior:** User explicitly chooses which of their pets to match with. No auto-detection. The `matches` table already exists with `(like_id_a < like_id_b)` constraint.

## Context & Decisions

| Decision | Rationale | Source |
|----------|-----------|--------|
| Remove auto-detection from `create_like` | Auto-detection caused duplicate matches; user control prevents ambiguity | Bug report + user preference |
| Add optional `reciprocal_pet_uuid` to `CreateLike` | User selects reciprocal pet in UI popup; backend uses it explicitly | User design |
| Show match dialog only on "like" direction, not "dislike" | Dislikes don't form matches; no reciprocal selection needed | UX logic |
| `potential_matches` returned by `check_interaction` | Frontend needs the list of reciprocal pets before user clicks "Like" | API design |
| Reuse `MatchPetInfo` for `potential_matches` elements | Same shape (pet_uuid, pet_name, species, primary_image_uuid); no need for a new type | DRY |
| No new migration needed | `matches` table already exists with correct schema | Existing state |

## Phase 1: Backend — Models [PENDING]

### 1.1 Add `reciprocal_pet_uuid` to `CreateLike` → `ref:plan-explicit-match`
- File: `src/models/likes.rs`, lines 108–113
- Add `pub reciprocal_pet_uuid: Option<PetUuid>` with `#[serde(default)]` for backward compatibility
- When `None` (default), like is created without match

### 1.2 Add `potential_matches` to `PetInteractionResponse` → `ref:plan-explicit-match`
- File: `src/models/likes.rs`, lines 184–189
- Add `pub potential_matches: Vec<MatchPetInfo>` with `#[serde(default)]`
- Reuse `MatchPetInfo` — same shape as what we need (pet_uuid, pet_name, species, primary_image_uuid)
- Populated by `get_pet_interaction()` — lists pets owned by current user that have been liked by the pet owner

### 1.3 Add unit tests for `CreateLike` deserialization → `ref:plan-explicit-match`
- File: `src/models/likes.rs`, in `mod test` (after line 326)
- `create_like_deserializes_with_reciprocal_pet_uuid` — verify new field deserializes correctly
- `create_like_deserializes_without_reciprocal_pet_uuid` — verify default `None` works

## Phase 2: Backend — Actions [PENDING]

### 2.1 Remove auto-detection block from `create_like()` → `ref:plan-explicit-match`
- File: `src/actions/likes.rs`, lines 59–98
- Delete the entire block that queries for reciprocal likes and creates matches automatically
- This includes the `use crate::schema::matches::dsl as matches` import and the `if let Ok(recip) = reciprocal` block

### 2.2 Add explicit match creation to `create_like()` → `ref:plan-explicit-match`
- File: `src/actions/likes.rs`, after line 57 (after inserting the like, before fetching)
- When `reciprocal_pet_uuid` is `Some`:
  1. Find the pet_id of `reciprocal_pet_uuid` and verify it belongs to **the current user** (`pets.user_id = *user_id`)
     — the reciprocal pet is one of the likemaster's own pets that the pet owner liked
  2. Find the like where `user_id = pet_owner_id` AND `pet_id = reciprocal_pet_id` AND `direction = Like`
     — this is the reciprocal like (the pet owner's like of the current user's pet)
  3. Create match record with the two like IDs (respecting `like_id_a < like_id_b` constraint)
- When `reciprocal_pet_uuid` is `None`: just create the like, no match
- Error handling: if reciprocal pet doesn't belong to current user or no reciprocal like exists → return 400 Bad Request with clear message

### 2.3 Rewrite `get_pet_interaction()` to return potential matches → `ref:plan-explicit-match`
- File: `src/actions/likes.rs`, lines 232–253
- Fetch pet owner's ID alongside pet_id: `let (pet_id, pet_owner_id): (PetId, UserId) = ...`
- Query for reciprocal likes: likes where `user_id = pet_owner_id` AND `direction = Like` AND `pet_id` is any of the current user's pets
- For each reciprocal like, build `MatchPetInfo` using existing `fetch_pet_with_image()` helper (no new helper needed)
- Return `PetInteractionResponse` with `potential_matches` populated

## Phase 3: Frontend — Models & API [PENDING]

### 3.1 Add `PetInteractionResponse` interface → `ref:plan-explicit-match`
- File: `frontend/app/models/pets.ts`, after `DiscoverQuery` (line ~115)
- Interface: `{ interacted: boolean; direction: 'like' | 'dislike' | null; potential_matches: MatchPetInfo[] }`
- Reuses existing `MatchPetInfo` interface

### 3.2 Update `likesApi.create()` → `ref:plan-explicit-match`
- File: `frontend/app/lib/api.ts`, lines 131–135
- Add optional `reciprocalPetUuid?: string` parameter
- Include `reciprocal_pet_uuid` in request body when provided

### 3.3 Update `likesApi.checkInteraction()` return type → `ref:plan-explicit-match`
- File: `frontend/app/lib/api.ts`, lines 152–155
- Change inline type to `Promise<PetInteractionResponse>` (the new interface)

## Phase 4: Frontend — Pet View Match Dialog [PENDING]

### 4.1 Add state for match dialog → `ref:plan-explicit-match`
- File: `frontend/app/pet-view/[pet_uuid].tsx`
- Add after line 68:
  ```typescript
  const [potentialMatches, setPotentialMatches] = useState<MatchPetInfo[]>([]);
  const [showMatchDialog, setShowMatchDialog] = useState(false);
  const [selectedReciprocalPet, setSelectedReciprocalPet] = useState<string | null>(null);
  const [isSubmittingLike, setIsSubmittingLike] = useState(false);
  ```

### 4.2 Update `fetchInteraction()` → `ref:plan-explicit-match`
- File: `frontend/app/pet-view/[pet_uuid].tsx`, lines 90–97
- Extract `potential_matches` from response and set `potentialMatches` state:
  ```typescript
  const res = await likesApi.checkInteraction(pet_uuid);
  setInteraction({ interacted: res.interacted, direction: res.direction });
  setPotentialMatches(res.potential_matches ?? []);
  ```

### 4.3 Rewrite `onLike()` → `ref:plan-explicit-match`
- File: `frontend/app/pet-view/[pet_uuid].tsx`, lines 119–131
- If `potentialMatches.length > 0`: show dialog (`setShowMatchDialog(true)`), return early
- If no potential matches: call new `submitLike()` directly (no dialog)
- Add new `submitLike(reciprocalPetUuid?: string)` function:
  ```typescript
  const submitLike = async (reciprocalPetUuid?: string) => {
    if (isSubmittingLike) return;
    setIsSubmittingLike(true);
    try {
      await likesApi.create(pet_uuid, 'like', reciprocalPetUuid);
      setInteraction({ interacted: true, direction: 'like' });
      setShowMatchDialog(false);
      if (reciprocalPetUuid) {
        Alert.alert('Match! 🎉', `You and ${pet?.owner.display_name || 'someone'} have a match!`);
      } else {
        Alert.alert('Liked!', `${pet?.name} has been liked.`);
      }
    } catch (err: any) {
      console.error('[Gallery] Like failed:', err);
      if (err?.response?.status === 409) {
        Alert.alert('Cannot Like', 'You cannot like your own pet.');
      } else if (err?.response?.status === 400) {
        Alert.alert('Error', 'Something went wrong with the match. Please try again.');
      }
    } finally {
      setIsSubmittingLike(false);
    }
  };
  ```

### 4.4 Add match selection dialog JSX → `ref:plan-explicit-match`
- File: `frontend/app/pet-view/[pet_uuid].tsx`, after the platform-specific return (before closing `</>`)

**Native version** (React Native):
```tsx
{showMatchDialog && (
  <View style={{ position: 'absolute', top: 0, left: 0, right: 0, bottom: 0, zIndex: 100 }}>
    {/* Backdrop */}
    <TouchableOpacity
      style={{ flex: 1, backgroundColor: 'rgba(0,0,0,0.5)' }}
      activeOpacity={1}
      onPress={() => !isSubmittingLike && setShowMatchDialog(false)}
    />
    {/* Bottom sheet */}
    <View style={{
      position: 'absolute', bottom: 0, left: 0, right: 0,
      backgroundColor: colors.background,
      borderTopLeftRadius: 20, borderTopRightRadius: 20,
      padding: 20, maxHeight: '60%',
    }}>
      <Text style={{ fontSize: 18, fontWeight: 700, color: colors.text, marginBottom: 4 }}>
        You have a match! 🎉
      </Text>
      <Text style={{ fontSize: 14, color: colors.grey, marginBottom: 16 }}>
        {pet?.owner.display_name || 'Someone'} liked one of your pets. Choose which pet to match with:
      </Text>

      {/* Pet selection list */}
      <ScrollView showsVerticalScrollIndicator={false} style={{ maxHeight: 240 }}>
        {potentialMatches.map((match) => (
          <TouchableOpacity
            key={match.pet_uuid}
            onPress={() => setSelectedReciprocalPet(match.pet_uuid)}
            style={{
              flexDirection: 'row', alignItems: 'center', padding: 12, borderRadius: 12,
              marginBottom: 8,
              backgroundColor: selectedReciprocalPet === match.pet_uuid
                ? accentSet.base : (isDarkColorScheme ? 'rgba(255,255,255,0.04)' : 'rgba(0,0,0,0.03)'),
            }}
          >
            {match.primary_image_uuid ? (
              <Image source={{ uri: getImageUrl(match.primary_image_uuid, 'thumbnail') }}
                style={{ width: 48, height: 48, borderRadius: 12, marginRight: 12 }} resizeMode="cover" />
            ) : (
              <View style={{ width: 48, height: 48, borderRadius: 12,
                backgroundColor: accentSet.bgSubtle || '#f0e0d8',
                justifyContent: 'center', alignItems: 'center', marginRight: 12 }}>
                <Ionicons name="paw" size={20} color={accentSet.base} />
              </View>
            )}
            <View style={{ flex: 1 }}>
              <Text style={{ fontSize: 15, fontWeight: 600, color: colors.text }}>{match.pet_name}</Text>
              <Text style={{ fontSize: 13, color: colors.grey }}>{match.species}</Text>
            </View>
            <Ionicons
              name={selectedReciprocalPet === match.pet_uuid ? 'checkmark-circle' : 'ellipse-outline'}
              size={24}
              color={selectedReciprocalPet === match.pet_uuid ? accentSet.base : colors.grey}
            />
          </TouchableOpacity>
        ))}
      </ScrollView>

      {/* Action buttons */}
      <View style={{ flexDirection: 'row', gap: 12, marginTop: 16 }}>
        <TouchableOpacity
          onPress={() => setShowMatchDialog(false)}
          style={{ flex: 1, paddingVertical: 14, borderRadius: 12,
            backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)',
            alignItems: 'center' }}>
          <Text style={{ fontSize: 15, fontWeight: 600, color: colors.grey }}>Skip</Text>
        </TouchableOpacity>
        <TouchableOpacity
          onPress={() => submitLike(selectedReciprocalPet ?? undefined)}
          disabled={!selectedReciprocalPet || isSubmittingLike}
          style={{ flex: 1, paddingVertical: 14, borderRadius: 12,
            backgroundColor: selectedReciprocalPet && !isSubmittingLike ? accentSet.base : (accentSet.base + '40'),
            alignItems: 'center',
            opacity: selectedReciprocalPet && !isSubmittingLike ? 1 : 0.5 }}>
          {isSubmittingLike ? (
            <ActivityIndicator color="#fff" size="small" />
          ) : (
            <Text style={{ fontSize: 15, fontWeight: 600, color: '#fff' }}>Confirm Match</Text>
          )}
        </TouchableOpacity>
      </View>
    </View>
  </View>
)}
```

**Web version** (HTML/Tailwind):
```tsx
{showMatchDialog && (
  <div style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, zIndex: 100 }}>
    {/* Backdrop */}
    <div className="fixed inset-0 bg-black/50" onClick={() => !isSubmittingLike && setShowMatchDialog(false)} />
    {/* Bottom sheet */}
    <div className="fixed bottom-0 left-0 right-0 bg-[colors.background] rounded-t-2xl p-5 max-h-[60%]">
      <h3 className="text-lg font-bold mb-1" style={{ color: colors.text }}>You have a match! 🎉</h3>
      <p className="text-sm mb-4" style={{ color: colors.grey }}>
        {pet?.owner.display_name || 'Someone'} liked one of your pets. Choose which pet to match with:
      </p>

      {/* Pet selection list */}
      <div className="overflow-y-auto max-h-60 mb-4">
        {potentialMatches.map((match) => (
          <div
            key={match.pet_uuid}
            onClick={() => setSelectedReciprocalPet(match.pet_uuid)}
            className="flex items-center p-3 rounded-xl mb-2 cursor-pointer"
            style={{
              backgroundColor: selectedReciprocalPet === match.pet_uuid
                ? accentSet.base : (isDarkColorScheme ? 'rgba(255,255,255,0.04)' : 'rgba(0,0,0,0.03)'),
            }}
          >
            {match.primary_image_uuid ? (
              <img src={getImageUrl(match.primary_image_uuid, 'thumbnail')} alt=""
                className="w-12 h-12 rounded-lg mr-3 object-cover" />
            ) : (
              <div className="w-12 h-12 rounded-lg mr-3 flex items-center justify-center"
                style={{ backgroundColor: accentSet.bgSubtle || '#f0e0d8' }}>
                <Ionicons name="paw" size={20} color={accentSet.base} />
              </div>
            )}
            <div className="flex-1">
              <div className="font-semibold text-sm" style={{ color: colors.text }}>{match.pet_name}</div>
              <div className="text-xs" style={{ color: colors.grey }}>{match.species}</div>
            </div>
            <Ionicons
              name={selectedReciprocalPet === match.pet_uuid ? 'checkmark-circle' : 'ellipse-outline'}
              size={24}
              color={selectedReciprocalPet === match.pet_uuid ? accentSet.base : colors.grey}
            />
          </div>
        ))}
      </div>

      {/* Action buttons */}
      <div className="flex gap-3">
        <button onClick={() => setShowMatchDialog(false)}
          className="flex-1 py-3 rounded-xl font-semibold text-sm"
          style={{ backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)', color: colors.grey }}>
          Skip
        </button>
        <button onClick={() => submitLike(selectedReciprocalPet ?? undefined)} disabled={!selectedReciprocalPet || isSubmittingLike}
          className="flex-1 py-3 rounded-xl font-semibold text-sm text-white"
          style={{
            backgroundColor: selectedReciprocalPet && !isSubmittingLike ? accentSet.base : (accentSet.base + '40'),
            opacity: selectedReciprocalPet && !isSubmittingLike ? 1 : 0.5,
          }}>
          {isSubmittingLike ? '...' : 'Confirm Match'}
        </button>
      </div>
    </div>
  </div>
)}
```

## Phase 5: Tests [PENDING]

### 5.1 Replace `create_like_detects_match` → `ref:plan-explicit-match`
- File: `tests/integration/discover.rs`, lines 370–436
- New test: `create_like_with_reciprocal_pet_creates_match`
  ```
  1. User1 creates pet (pet1_uuid)
  2. User2 creates pet (pet2_uuid)
  3. User1 likes pet2_uuid → like created, no match (no reciprocal like exists yet)
  4. User2 likes pet1_uuid WITH reciprocal_pet_uuid=pet2_uuid
     - pet2_uuid is owned by User2 (the pet_owner), so validation passes
     - finds the like where user_id=User2 AND pet_id=pet2's_id (the like from step 3)
     - creates match between the two likes
  5. Verify match count = 1
  ```
  Key: `reciprocal_pet_uuid` is one of User2's own pets (pet2_uuid) that User1 has liked. The backend finds the like record where User1 liked pet2, and pairs it with the new like where User2 likes pet1.

### 5.2 Add `create_like_without_reciprocal_pet_no_match` → `ref:plan-explicit-match`
- File: `tests/integration/discover.rs`
- User1 likes User2's pet without `reciprocal_pet_uuid` (no prior reciprocal like needed)
- Verify match count = 0 (no auto-detection)

### 5.3 Add `check_interaction_returns_potential_matches` → `ref:plan-explicit-match`
- File: `tests/integration/discover.rs`
- User1 creates two pets (pet1a, pet1b), User2 creates one pet (pet2)
- User1 likes pet2 → like created
- User2 calls `check_interaction` for pet1a (one of User1's pets)
- Verify `potential_matches` contains pet2 (the pet User1 liked, which User2 can match with)

### 5.4 Add `create_like_with_invalid_reciprocal_pet_returns_error` → `ref:plan-explicit-match`
- File: `tests/integration/discover.rs`
- User2 creates a pet, User1 likes it
- User2 tries to like User1's pet with a fake `reciprocal_pet_uuid` (one that doesn't belong to User2)
- Verify 400 Bad Request response

### 5.5 No change needed for `create_like_success` → `ref:plan-explicit-match`
- File: `tests/integration/discover.rs`, lines 333–368
- Test sends a like without `reciprocal_pet_uuid` and verifies 201 Created — still passes

## Notes

- The `matches` table already has the correct schema: `(like_id_a < like_id_b)` constraint, foreign keys to `likes(id)`, `matched_at` timestamp
- `#[serde(default)]` on `reciprocal_pet_uuid` ensures backward compatibility
- Error cases in `create_like` with invalid `reciprocal_pet_uuid` should return 400 Bad Request (not 500)
- The `potential_matches` list may be empty if the pet owner hasn't liked any of the current user's pets — like button works normally without a dialog

## Affected Files

| File | Change |
|------|--------|
| `src/models/likes.rs` | Add `reciprocal_pet_uuid` to `CreateLike`; add `potential_matches: Vec<MatchPetInfo>` to `PetInteractionResponse`; add 2 unit tests |
| `src/actions/likes.rs` | Remove auto-detection from `create_like()`; add explicit match creation when `reciprocal_pet_uuid` provided; rewrite `get_pet_interaction()` to return potential matches using existing `fetch_pet_with_image()` |
| `frontend/app/models/pets.ts` | Add `PetInteractionResponse` interface |
| `frontend/app/lib/api.ts` | Update `likesApi.create()` to accept optional `reciprocalPetUuid`; update `checkInteraction()` return type |
| `frontend/app/pet-view/[pet_uuid].tsx` | Add match dialog state; rewrite `onLike()` to show dialog; add `submitLike()` helper; add match selection dialog JSX (native + web) |
| `tests/integration/discover.rs` | Replace `create_like_detects_match`; add 4 new integration tests |
