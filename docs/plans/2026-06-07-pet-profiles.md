# Pet Profiles

## Overview

Add a pet profile system allowing users to create, manage, and search multiple pet profiles per user. Each pet has personality traits (predefined curated list), basic info (species, breed, etc.), and optional avatar images.

## Status: Planning

## Database Schema

### `pets` table

```sql
CREATE TABLE pets (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    species VARCHAR(50) NOT NULL,
    breed VARCHAR(200),
    date_of_birth DATE,
    gender VARCHAR(10),  -- male/female/neuter/spayed/etc.
    weight DECIMAL(8,2),  -- in kg
    color_markings VARCHAR(200),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Updated at trigger (same pattern as profiles)
CREATE OR REPLACE FUNCTION update_pets_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_pets_updated_at
    BEFORE UPDATE ON pets
    FOR EACH ROW EXECUTE FUNCTION update_pets_updated_at();
```

### `personality_traits` table

```sql
CREATE TABLE personality_traits (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed with curated traits
INSERT INTO personality_traits (name) VALUES
    ('playful'), ('calm'), ('energetic'), ('affectionate'),
    ('independent'), ('curious'), ('loyal'), ('gentle'),
    ('stubborn'), ('social'), ('shy'), ('protective'),
    ('vocally expressive'), ('food motivated'), ('smart');
```

### `pet_personality_traits` junction table

```sql
CREATE TABLE pet_personality_traits (
    pet_id INTEGER NOT NULL REFERENCES pets(id) ON DELETE CASCADE,
    trait_id INTEGER NOT NULL REFERENCES personality_traits(id) ON DELETE CASCADE,
    PRIMARY KEY (pet_id, trait_id)
);
```

## API Surface

### Trait endpoints (public)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/public/pets/traits` | none | List all personality traits (for create/filter) |

### Pet endpoints (authenticated owner)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/user/pets` | RoleUser | Create a new pet |
| GET | `/api/user/pets` | RoleUser | List user's pets (with `?species=dog&traits=playful` filters) |
| GET | `/api/user/pets/{pet_id}` | RoleUser | Get one of user's pets (404 if not owned) |
| PATCH | `/api/user/pets/{pet_id}` | RoleUser | Update pet (partial update) |
| DELETE | `/api/user/pets/{pet_id}` | RoleUser | Delete a pet (hard delete) |

### Pet endpoints (public)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/public/pets/{pet_id}` | none | Public pet view (without owner info) |

### Request/Response models

**CreatePet:**
```rust
pub struct CreatePet {
    pub name: String,
    pub species: String,
    pub breed: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub weight: Option<f64>,
    pub color_markings: Option<String>,
    pub description: Option<String>,
    pub traits: Vec<String>,  // trait names to assign
}
```

**UpdatePet:**
```rust
pub struct UpdatePet {
    pub name: Option<String>,
    pub species: Option<String>,
    pub breed: Option<Option<String>>,  // None = omit, Some(None) = clear
    pub date_of_birth: Option<Option<NaiveDate>>,
    pub gender: Option<Option<String>>,
    pub weight: Option<Option<f64>>,
    pub color_markings: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub traits: Option<Vec<String>>,  // None = omit, Some(vec) = replace all traits
}
```

**PublicPet:**
```rust
pub struct PublicPet {
    pub id: PetId,
    pub name: String,
    pub species: String,
    pub breed: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub weight: Option<f64>,
    pub color_markings: Option<String>,
    pub description: Option<String>,
    pub traits: Vec<PetTrait>,  // { id, name }
}
```

**PetTrait:**
```rust
pub struct PetTrait {
    pub id: TraitId,
    pub name: String,
}
```

## Implementation Steps

### 1. Migration
- Create migration directory with up.sql (pets, personality_traits, pet_personality_traits) and down.sql
- Run migration, then `diesel print-schema` to generate schema.rs

### 2. Models
- Newtypes: `PetId`, `TraitId` (DieselNewType)
- Pet model: `Pet`, `CreatePet`, `UpdatePet`, `PublicPet`, `PetTrait`
- Trait models: `PersonalityTrait`, `PetPersonalityTrait`
- Custom deserializer on `UpdatePet` for partial update semantics (same pattern as UpdateProfile)

### 3. Actions (business logic)
- `create_pet(user_id, create, conn)` — creates pet, then inserts trait junctions
- `get_pet(pet_id, conn)` — fetches pet with traits
- `list_pets(user_id, filters, conn)` — fetches user's pets with optional species/traits filters
- `update_pet(pet_id, user_id, updates, conn)` — partial update, replaces traits if provided
- `delete_pet(pet_id, user_id, conn)` — ownership check + hard delete
- `list_traits(conn)` — list all personality traits
- `get_public_pet(pet_id, conn)` — public view (no owner info)

### 4. Routes (HTTP handlers)
- Wire all endpoints with utoipa path annotations
- Ownership enforcement: GET/PATCH/DELETE verify `user_id` matches owner
- Public pet endpoint: no ownership check, strips owner info from response
- `web::block` for all DB calls, `#[tracing::instrument]` on all handlers

### 5. Wiring
- Register routes in `src/lib.rs` under existing scopes
- Register OpenAPI schemas

### 6. Tests
- Integration tests: create, read, partial update (null-clear), delete with ownership check, public pet, trait filtering
- Unit tests: newtype validators, UpdatePet deserializer, trait search logic

## Notes
- Traits are assigned by name on create (looked up from personality_traits table)
- On PATCH, `traits` field replaces all existing traits for the pet (not additive)
- Species filter is case-insensitive ILIKE or lowercase comparison
- Trait search: JOIN pet_personality_traits + personality_traits, filter by trait name
- No avatar upload in v1 — just store URL string if needed later
