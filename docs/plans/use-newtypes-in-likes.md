# Use PetName/PetSpecies newtypes in likes models

## Goal
Replace raw `String` fields in `MatchPetInfo` with the proper newtype fields (`PetName`, `PetSpecies`) from `src/models/pets.rs`.

## Background
`PetName` and `PetSpecies` derive `DieselNewType` (wrapping `String`). Since the DB columns are `Varchar`, diesel loads them directly as the newtype — no manual parsing needed. The `Pet` struct (line 341 in `pets.rs`) proves this: it has `name: PetName` and `species: PetSpecies` loaded via `Queryable`.

## Changes

### 1. `src/models/likes.rs` (lines 9, 199-204)
**Import** — add `PetName`, `PetSpecies` to the import from `super::pets`:
```rust
use super::pets::{PetId, PetName, PetSpecies, PetUuid, PublicPetOwner};
```

**MatchPetInfo struct** — change field types:
```rust
pub struct MatchPetInfo {
    pub pet_uuid: PetUuid,
    pub pet_name: PetName,        // was: String
    pub species: PetSpecies,      // was: String
    pub primary_image_uuid: Option<String>,
}
```

### 2. `src/actions/likes.rs` (lines 313-345)
**Import** — add `PetName`, `PetSpecies` to imports from `crate::models::pets`.

**fetch_pet_with_image return type** — change return tuple:
```rust
// Before:
fn fetch_pet_with_image(
    pet_id: &PetId,
    conn: &mut DbConnection,
) -> Result<(PetUuid, String, String, Option<uuid::Uuid>), DomainError>

// After:
fn fetch_pet_with_image(
    pet_id: &PetId,
    conn: &mut DbConnection,
) -> Result<(PetUuid, PetName, PetSpecies, Option<uuid::Uuid>), DomainError>
```

**Inside fetch_pet_with_image** — load newtypes directly from diesel (no manual parsing):
```rust
// Before:
let (pet_uuid_raw, pet_name, species_raw): (uuid::Uuid, String, String) =
    pets::pets
        .select((pets::pet_uuid, pets::name, pets::species))
        .filter(pets::id.eq(*pet_id))
        .first(conn)
        .map_err(DomainError::from)?;

let pet_uuid: PetUuid = PetUuid::try_from(pet_uuid_raw.to_string())...;
// ... PetName::new(pet_name)... / PetSpecies::new(species_raw)...

// After:
let (pet_uuid_raw, pet_name, species): (uuid::Uuid, PetName, PetSpecies) =
    pets::pets
        .select((pets::pet_uuid, pets::name, pets::species))
        .filter(pets::id.eq(*pet_id))
        .first(conn)
        .map_err(DomainError::from)?;

let pet_uuid: PetUuid = PetUuid::try_from(pet_uuid_raw.to_string())...;
// pet_name and species are already the correct newtypes — pass through directly
```

### 3. `src/actions/likes.rs` (lines ~267-292, list_matches_with_pets)
The code that constructs `MatchPetInfo` will now receive `PetName`/`PetSpecies` directly from `fetch_pet_with_image` instead of `String`, so no changes needed to the struct construction — just pass the values through.

## Validation
- `cargo make lint-check` — fmt + clippy -D warnings ✅
- `cargo make test` — 50 unit tests pass ✅
- `tsc --noEmit` — frontend typecheck ✅

## Additional Changes (beyond original plan)
- Added `PetImageUuid` newtype in `src/models/pets.rs` (UUID newtype for pet image IDs)
- Updated `PetImage.uuid` from `Uuid` → `PetImageUuid`
- Updated `PublicPetImage.uuid` from `Uuid` → `PetImageUuid`
- Updated `PublicPetOwner.display_name` from `Option<String>` → `Option<DisplayName>`
- Updated `MatchWithPets.other_owner_name` from `Option<String>` → `Option<DisplayName>`
- Updated all query sites to load newtypes directly from diesel
