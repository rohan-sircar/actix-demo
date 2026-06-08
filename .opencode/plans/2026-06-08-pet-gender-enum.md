# Convert PetGender to PostgreSQL Enum

## Goal
Replace the `PetGender` String validator newtype with a proper PostgreSQL enum type (`pet_gender`) following the same pattern as `JobStatus`. Variants: `male`, `female`, `unspecified`.

## Changes

### 1. Migration: `migrations/2026-06-07-000000_create_pets/up.sql`

Add enum type definition before the `pets` table:
```sql
CREATE TYPE pet_gender AS ENUM ('male', 'female', 'unspecified');
```

Change the `gender` column from `VARCHAR(10)` to use the enum type:
```sql
gender pet_gender,
```

### 2. Migration: `migrations/2026-06-07-000000_create_pets/down.sql`

Add before dropping the table:
```sql
DROP TYPE IF EXISTS pet_gender;
```

### 3. Schema: `src/schema.rs`

Regenerate by running `diesel migration run` then `diesel generate-schema`. Diesel CLI will auto-generate:
```rust
pub mod sql_types {
    // ... existing types ...

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pet_gender"))]
    pub struct PetGender;
}
```

And the `pets` table column will change from:
```rust
#[max_length = 10]
gender -> Nullable<Varchar>,
```
to:
```rust
gender -> Nullable<PetGender>,
```

### 4. Models: `src/models/pets.rs`

#### Replace `PetGender` validator newtype with enum:

Delete the existing `PetGender` struct and its impl block:
```rust
// TODO should be enum
#[derive(Validator, Debug, Clone, DieselNewType, PartialEq, Eq, ToSchema)]
#[validator(line(char_length(max = 10)))]
pub struct PetGender(String);

impl PetGender {
    pub fn new(value: String) -> Result<Self, String> {
        Self::parse_string(&value).map_err(|e| e.to_string())
    }
    pub fn inner(&self) -> &str {
        &self.0
    }
}
```

Replace with:
```rust
#[derive(
    DbEnum,
    Debug,
    Clone,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    ToSchema,
)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::PetGender"]
pub enum PetGender {
    Male,
    Female,
    Unspecified,
}
```

#### Update `CreatePet` model:
No change — `pub gender: Option<PetGender>` stays the same. Diesel `Insertable` handles the enum automatically.

#### Update `UpdatePet` deserializer:
Change the gender parsing from:
```rust
"gender" => {
    if val.is_null() {
        gender = Some(None);
    } else if let Some(s) = val.as_str() {
        gender = Some(Some(
            PetGender::new(s.to_string())
                .map_err(serde::de::Error::custom)?,
        ));
    }
}
```

To:
```rust
"gender" => {
    if val.is_null() {
        gender = Some(None);
    } else if let Some(s) = val.as_str() {
        let parsed: Result<PetGender, _> = s.parse();
        gender = Some(Some(
            parsed.map_err(serde::de::Error::custom)?,
        ));
    }
}
```

Invalid gender strings (not matching `male`/`female`/`unspecified`) will return a 400 error.

#### Update `Pet` model:
No change — `pub gender: Option<PetGender>` stays the same. Diesel `Queryable` handles the enum automatically.

#### Update `PublicPet` model:
No change — `pub gender: Option<String>` stays as `Option<String>` for the public response. The `From<(&Pet, Vec<PetTrait>)>` impl already converts via `pet.gender.as_ref().map(|g| g.0.clone())` — this will need updating to use the enum's `Display` or a match:
```rust
gender: pet.gender.as_ref().map(|g| match g {
    PetGender::Male => "male",
    PetGender::Female => "female",
    PetGender::Unspecified => "unspecified",
}),
```

Actually, since `PetGender` now implements `Serialize` via `#[serde(rename_all = "lowercase")]`, we could also just return `Option<PetGender>` in `PublicPet`. But that would change the API response shape (from string to enum-like object in JSON). Let me check what `JobStatus` does in the public response...

Looking at `JobCount` in misc.rs, `JobStatus` is returned directly as the enum type. For consistency and to keep the public API simple, `PublicPet.gender` should stay as `Option<String>` with the lowercase variant name. The match approach above is correct.

#### Update unit test:
Change `gender: Some(PetGender("male".to_string()))` to `gender: Some(PetGender::Male)`.

### 5. Actions: `src/actions/pets.rs`

No changes needed — all action functions accept/return the model types which already have `Option<PetGender>`. Diesel handles the enum-to-database conversion automatically via `DbEnum`.

### 6. Routes: `src/routes/pets.rs`

No changes needed — `gender` is in the request body (`CreatePet` / `UpdatePet`), not a path parameter. The serde deserializer handles the string-to-enum conversion.

### 7. Route Registration: `src/lib.rs`

Add `PetGender` to OpenAPI schema components:
```rust
models::pets::PetGender,
```

### 8. Tests: `tests/integration/pets.rs`

No changes needed — tests don't set gender in any of the pet creation/update payloads. The field is `Option<PetGender>` and defaults to `None` when not provided.

### 9. Edge Cases & Considerations

#### a) Backward compatibility
Existing clients sending arbitrary strings (e.g., "M", "F", "Male", "female", "unknown") will now get 400 errors. Since this is a fresh feature with no deployed clients yet, this is acceptable.

#### b) Database enum vs VARCHAR with CHECK
Using a PostgreSQL enum type provides:
- Strong typing at the database level
- Diesel `DbEnum` integration for automatic conversion
- Self-documenting schema
- Smaller storage (4 bytes vs variable-length string)

The tradeoff is that adding a new variant requires a migration (`ALTER TYPE pet_gender ADD VALUE '...'`). With a VARCHAR + CHECK constraint, new values would be accepted without schema changes. For a fixed set like gender, the enum approach is appropriate.

#### c) Case sensitivity
`#[serde(rename_all = "lowercase")]` means the API accepts only lowercase: `"male"`, `"female"`, `"unspecified"`. This is consistent with the PostgreSQL enum values.

#### d) `PetGender` in diesel-derive-enum
The `DbEnum` derive requires:
- `#[ExistingTypePath = "crate::schema::sql_types::PetGender"]` — tells Diesel this maps to a custom PG type
- `#[serde(rename_all = "lowercase")]` — controls JSON serialization
- `to_string_representation()` and `try_from_storage()` are auto-implemented by `DbEnum`

#### e) Nullable handling
Since `gender` is nullable in the database, all model fields use `Option<PetGender>`. Diesel `Queryable` correctly maps SQL NULL to `None`.

## Implementation Order
1. Update migration files (up.sql + down.sql)
2. Regenerate schema (diesel migration run + diesel generate-schema)
3. Replace PetGender newtype with enum in models/pets.rs
4. Update UpdatePet deserializer for gender parsing
5. Update PublicPet gender conversion in From impl
6. Add PetGender to OpenAPI schema components in lib.rs
7. Update unit test for PetGender
8. Run `cargo test` to verify
