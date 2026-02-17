# Pet Profiles Feature Implementation for Pet Dating App

## Objective

Implement a comprehensive pet profile system that allows users to create and manage multiple pet profiles (dogs and cats) within a pet dating application. This feature will enable pet owners to showcase their pets and find suitable matches.

## Overview

Users can have multiple pet profiles associated with their account, allowing them to manage dogs, cats, and other pets individually. Each pet profile contains detailed information relevant to pet matching and dating functionality.

## Suggested Profile Fields for Dating App

### Core Pet Information

pet_name (String) - Your pet's name
pet_type (Enum: "dog" | "cat") - Species type
breed (String) - Dog/cat breed
age (Integer) - Age in years
weight_kg (Decimal) - Weight in kilograms
gender (Enum: "male" | "female") - Pet's gender
size (Enum: "toy" | "small" | "medium" | "large" | "giant") - Size category
color (String) - Coat color
coat_type (Enum: "short" | "medium" | "long" | "curly" | "wire" | "hairless") - Fur type

### Personality & Interests

bio (Text) - Pet's personality description
personality_traits - e.g., ["playful", "loyal", "calm", "energetic"]
good_with_dogs (Boolean) - For dogs
good_with_cats (Boolean) - For cats
good_with_kids (Boolean)
house_trained (Boolean)
vaccinated (Boolean)
spayed_neutered (Boolean)
microchipped (Boolean)
Activities & Preferences
favorite_activities - e.g., ["fetch", "hiking", "cuddling", "dog parks"]
likes - e.g., ["toys", "treats", "balls", "chasing"]
dislikes - e.g., ["thunder", "vacuum", "strangers"]
energy_level (Enum: "low" | "medium" | "high" | "extreme") - Activity needs
trainability (Enum: "easy" | "moderate" | "challenging") - Ease of training
barking_level (Enum: "quiet" | "moderate" | "loud") - For dogs

### Owner Information

owner_name (String) - Owner's first name
location (String) - City/neighborhood
address (String) - Full address (for distance matching)
lat (Float) - Latitude for location
lng (Float) - Longitude for location
availability (JSONB) - e.g., {"weekdays": "evening", "weekends": "all_day"}

### Media

profile_pictures - Array of picture objects with URL and order
videos - Array of video URLs
video_bio (String) - Video introduction link

## Special Considerations

special_needs (Boolean) - Has special requirements
special_needs_description (Text) - Details if applicable
adoption_status (Enum: "adoptable" | "foster" | "available") - For shelters
shelter_name (String) - If applicable

## Database Design

### [`migrations/2026-02-11-133839_add_user_profile/up.sql`](migrations/2026-02-11-133839_add_user_profile/up.sql:1)

```sql
-- Create pet_types enum
CREATE TYPE pet_type AS ENUM ('dog', 'cat');
CREATE TYPE gender_type AS ENUM ('male', 'female');
CREATE TYPE size_type AS ENUM ('toy', 'small', 'medium', 'large', 'giant');
CREATE TYPE coat_type AS ENUM ('short', 'medium', 'long', 'curly', 'wire', 'hairless');
CREATE TYPE energy_level_type AS ENUM ('low', 'medium', 'high', 'extreme');
CREATE TYPE trainability_type AS ENUM ('easy', 'moderate', 'challenging');
CREATE TYPE barking_level_type AS ENUM ('quiet', 'moderate', 'loud');
CREATE TYPE adoption_status_type AS ENUM ('adoptable', 'foster', 'available');

-- Create pet_profiles table (multiple pets per user)
CREATE TABLE pet_profiles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Core pet info
    pet_name VARCHAR(100) NOT NULL,
    pet_type pet_type NOT NULL,
    breed VARCHAR(100) NOT NULL,
    age INTEGER NOT NULL CHECK (age >= 0 AND age <= 30),
    weight_kg DECIMAL(5,2) NOT NULL CHECK (weight_kg > 0),
    gender gender_type NOT NULL,
    size size_type,
    color VARCHAR(50),
    coat_type coat_type,

    -- Personality
    bio TEXT,
    personality_traits TEXT[] DEFAULT '{}',
    good_with_dogs BOOLEAN,
    good_with_cats BOOLEAN,
    good_with_kids BOOLEAN,
    house_trained BOOLEAN,
    vaccinated BOOLEAN,
    spayed_neutered BOOLEAN,
    microchipped BOOLEAN,

    -- Activities
    favorite_activities TEXT[] DEFAULT '{}',
    likes TEXT[] DEFAULT '{}',
    dislikes TEXT[] DEFAULT '{}',
    energy_level energy_level_type,
    trainability trainability_type,
    barking_level barking_level_type,

    -- Owner info
    owner_name VARCHAR(100) NOT NULL,
    location VARCHAR(100) NOT NULL,
    address TEXT,
    lat DECIMAL(10,8),
    lng DECIMAL(11,8),

    -- Special considerations
    special_needs BOOLEAN DEFAULT false,
    special_needs_description TEXT,
    adoption_status adoption_status_type,
    shelter_name VARCHAR(100),

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_pet_profiles_user_id ON pet_profiles(user_id);
-- CREATE INDEX idx_pet_profiles_breed ON pet_profiles(breed);
-- CREATE INDEX idx_pet_profiles_pet_type ON pet_profiles(pet_type);
-- CREATE INDEX idx_pet_profiles_size ON pet_profiles(size);
-- CREATE INDEX idx_pet_profiles_gender ON pet_profiles(gender);
-- CREATE INDEX idx_pet_profiles_location ON pet_profiles(location);
-- CREATE INDEX idx_pet_profiles_good_with_dogs ON pet_profiles(good_with_dogs);
-- CREATE INDEX idx_pet_profiles_good_with_cats ON pet_profiles(good_with_cats);
-- CREATE INDEX idx_pet_profiles_good_with_kids ON pet_profiles(good_with_kids);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_pet_profiles_updated_at
    BEFORE UPDATE ON pet_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

## Model Design

### [`src/models/pet_profiles.rs`](src/models/pet_profiles.rs:1)

```rust
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::schema::pet_profiles;
use crate::models::users::UserId;

#[derive(Debug, Clone, Deserialize, Serialize, DieselNewType)]
pub struct PetProfileId(i32);

impl PetProfileId {
    pub fn as_uint(&self) -> u32 {
        self.0.try_into().unwrap()
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, diesel::Queryable, diesel::FromSqlRow)]
#[diesel(postgres_type_name = "pet_type")]
#[serde(rename_all = "lowercase")]
pub enum PetType {
    Dog,
    Cat,
}

// ... other enums (GenderType, SizeType, CoatType, etc.) ...

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[diesel(table_name = pet_profiles)]
pub struct PetProfile {
    pub id: PetProfileId,
    pub user_id: UserId,
    pub profile_name: String,
    pub pet_name: String,
    pub pet_type: PetType,
    pub breed: String,
    pub age: i32,
    pub weight_kg: f64,
    pub gender: GenderType,
    pub size: Option<SizeType>,
    pub color: Option<String>,
    pub coat_type: Option<CoatType>,
    pub bio: Option<String>,
    pub personality_traits: Option<Vec<String>>,
    pub good_with_dogs: Option<bool>,
    pub good_with_cats: Option<bool>,
    pub good_with_kids: Option<bool>,
    pub house_trained: Option<bool>,
    pub vaccinated: Option<bool>,
    pub spayed_neutered: Option<bool>,
    pub microchipped: Option<bool>,
    pub favorite_activities: Option<Vec<String>>,
    pub likes: Option<Vec<String>>,
    pub dislikes: Option<Vec<String>>,
    pub energy_level: Option<EnergyLevelType>,
    pub trainability: Option<TrainabilityType>,
    pub barking_level: Option<BarkingLevelType>,
    pub owner_name: String,
    pub location: String,
    pub address: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub special_needs: bool,
    pub special_needs_description: Option<String>,
    pub adoption_status: Option<AdoptionStatusType>,
    pub shelter_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_profiles)]
pub struct NewPetProfile {
    pub user_id: UserId,
    pub profile_name: String,
    pub pet_name: String,
    pub pet_type: PetType,
    pub breed: String,
    pub age: i32,
    pub weight_kg: f64,
    pub gender: GenderType,
    // ... other fields
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = pet_profiles)]
pub struct UpdatePetProfile {
    pub profile_name: Option<String>,
    pub pet_name: Option<String>,
    pub pet_type: Option<PetType>,
    // ... other optional fields
}
```

## CRUD Operations

### [`src/actions/pet_profiles.rs`](src/actions/pet_profiles.rs:1)

```rust
use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::pet_profiles::{
    PetProfile, PetProfileId, NewPetProfile, UpdatePetProfile
};
use crate::models::users::UserId;
use crate::types::DbConnection;

// Get all pet profiles for a user
pub fn get_user_pet_profiles(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<PetProfile>, DomainError> {
    use crate::schema::pet_profiles::dsl as profiles;

    let _ = tracing::info!("Getting pet profiles for user {user_id}");

    profiles::pet_profiles
        .filter(profiles::user_id.eq(user_id))
        .order_by(profiles::is_primary.desc().then(profiles::created_at.asc()))
        .load::<PetProfile>(conn)
        .map_err(|err| DomainError::new_internal_error(format!("Failed to retrieve profiles: {err}")))
}


// Create new pet profile with automatic primary profile management
pub fn create_pet_profile(
    new_profile: NewPetProfile,
    conn: &mut DbConnection,
) -> Result<PetProfile, DomainError> {
    use crate::schema::pet_profiles::dsl as profiles;

    let _ = tracing::info!("Creating pet profile for user {}", new_profile.user_id);

    conn.transaction(|conn| {
        // If marking as primary, ensure it's the only primary profile
        if new_profile.is_primary.unwrap_or(false) {
            diesel::update(profiles::pet_profiles)
                .filter(profiles::user_id.eq(new_profile.user_id))
                .set(profiles::is_primary.eq(false))
                .execute(conn)?;
        }

        // Insert the new profile
        let profile = diesel::insert_into(profiles::pet_profiles)
            .values(&new_profile)
            .returning(PetProfile::as_returning())
            .get_result(conn)
            .map_err(|err| {
                let _ = tracing::error!("Failed to create profile: {err}");
                err
            })?;

        Ok(profile)
    })
    .map_err(|err| DomainError::new_internal_error(format!("Failed to create profile: {err}")))
}

// Update pet profile with ownership validation
pub fn update_pet_profile(
    profile_id: &PetProfileId,
    user_id: &UserId,
    update_data: UpdatePetProfile,
    conn: &mut DbConnection,
) -> Result<PetProfile, DomainError> {
    use crate::schema::pet_profiles::dsl as profiles;

    let _ = tracing::info!("Updating pet profile {profile_id} for user {user_id}");

    conn.transaction(|conn| {
        // Verify ownership
        let owned = profiles::pet_profiles
            .filter(profiles::id.eq(profile_id))
            .filter(profiles::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)? == 1;

        if !owned {
            return Err(DomainError::new_forbidden_error(
                "You can only update your own pet profiles".to_string()
            ));
        }

        // Handle primary profile updates
        if update_data.is_primary == Some(true) {
            diesel::update(profiles::pet_profiles)
                .filter(profiles::user_id.eq(user_id))
                .filter(profiles::id.ne(profile_id))
                .set(profiles::is_primary.eq(false))
                .execute(conn)?;
        }

        // Perform the update
        diesel::update(profiles::pet_profiles.find(profile_id))
            .set(&update_data)
            .returning(PetProfile::as_returning())
            .get_result(conn)
            .map_err(|err| DomainError::new_internal_error(format!("Failed to update profile: {err}")))
    })
}
```

## API Routes

### [`src/routes/pet_profiles.rs`](src/routes/pet_profiles.rs:1)

```rust
use actix_web::{web, HttpRequest, HttpResponse};
use crate::{errors::DomainError, AppData};
use crate::utils::cookie_auth::CookieAuth;
use crate::models::pet_profiles::{
    NewPetProfile, UpdatePetProfile, PetProfileId
};

// Get all pet profiles for authenticated user
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_my_pet_profiles(
    app_data: web::Data<AppData>,
    auth: CookieAuth,
) -> Result<HttpResponse, DomainError> {
    let user_id = get_user_id_from_auth(&auth, &app_data).await?;

    let profiles = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profiles::get_user_pet_profiles(&user_id, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(profiles))
}

// Create new pet profile
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn create_pet_profile(
    app_data: web::Data<AppData>,
    form: web::Json<NewPetProfile>,
    auth: CookieAuth,
) -> Result<HttpResponse, DomainError> {
    let user_id = get_user_id_from_auth(&auth, &app_data).await?;

    // Validate ownership
    if form.user_id != user_id {
        return Err(DomainError::new_forbidden_error(
            "You can only create profiles for yourself".to_string()
        ));
    }

    let profile = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profiles::create_pet_profile(form.0, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Created().json(profile))
}

// Update pet profile with ownership verification
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn update_pet_profile(
    app_data: web::Data<AppData>,
    profile_id: web::Path<PetProfileId>,
    update_data: web::Json<UpdatePetProfile>,
    auth: CookieAuth,
) -> Result<HttpResponse, DomainError> {
    let user_id = get_user_id_from_auth(&auth, &app_data).await?;

    let profile = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profiles::update_pet_profile(&profile_id, &user_id, update_data.0, &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(profile))
}
```

## Image Storage and Ordering

### Database Design for Images

```sql
-- Create separate image table
CREATE TABLE pet_profile_images (
    id SERIAL PRIMARY KEY,
    pet_profile_id INTEGER NOT NULL REFERENCES pet_profiles(id) ON DELETE CASCADE,
    image_url TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT false,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_pet_profile_images_pet_profile_id ON pet_profile_images(pet_profile_id);
CREATE UNIQUE INDEX idx_pet_profile_images_primary ON pet_profile_images(pet_profile_id) WHERE is_primary = true;
```

### Rust Model for PetProfileImage

```rust
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::schema::pet_profile_images;
use crate::models::pet_profiles::PetProfileId;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct PetProfileImage {
    pub id: i32,
    pub pet_profile_id: PetProfileId,  // Uses PetProfileId newtype
    pub image_url: String,
    pub is_primary: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pet_profile_images)]
pub struct NewPetProfileImage {
    pub pet_profile_id: PetProfileId,  // Uses PetProfileId newtype
    pub image_url: String,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}
```

### Image Ordering Implementation

The `sort_order` column enables frontend reordering:

```rust
// In src/actions/pet_profile_images.rs

use crate::models::pet_profiles::PetProfileId;

// Get images ordered by sort_order
pub fn get_profile_images(
    profile_id: &PetProfileId,
    conn: &mut DbConnection,
) -> Result<Vec<PetProfileImage>, DomainError> {
    use crate::schema::pet_profile_images::dsl as images;

    let _ = tracing::info!("Getting images for pet profile {profile_id}");

    images::pet_profile_images
        .filter(images::pet_profile_id.eq(profile_id.0))
        .order_by(images::sort_order.asc())
        .load::<PetProfileImage>(conn)
        .map_err(|err| DomainError::new_internal_error(format!("Failed to retrieve images: {err}")))
}

// Update image ordering (drag-and-drop)
pub fn update_image_order(
    profile_id: &PetProfileId,
    image_ids_with_order: &[(i32, i32)],  // [(image_id, new_sort_order), ...]
    conn: &mut DbConnection,
) -> Result<(), DomainError> {
    use crate::schema::pet_profile_images::dsl as images;

    let _ = tracing::info!("Updating image order for profile {profile_id}");

    conn.transaction(|conn| {
        for (image_id, new_order) in image_ids_with_order {
            diesel::update(images::pet_profile_images.find(image_id))
                .set(images::sort_order.eq(new_order))
                .execute(conn)?;
        }
        Ok(())
    })
    .map_err(|err| DomainError::new_internal_error(format!("Failed to update image order: {err}")))
}

// Add image with primary flag handling
pub fn add_profile_image(
    new_image: NewPetProfileImage,
    conn: &mut DbConnection,
) -> Result<PetProfileImage, DomainError> {
    use crate::schema::pet_profile_images::dsl as images;

    conn.transaction(|conn| {
        // If marking as primary, demote others
        if new_image.is_primary == Some(true) {
            diesel::update(
                images::pet_profile_images.filter(
                    images::pet_profile_id.eq(new_image.pet_profile_id.0)
                )
            ).set(images::is_primary.eq(false)).execute(conn)?;
        }

        diesel::insert_into(images::pet_profile_images)
            .values(&new_image)
            .returning(PetProfileImage::as_returning())
            .get_result(conn)
    })
    .map_err(|err| DomainError::new_internal_error(format!("Failed to add image: {err}")))
}
```

### Frontend API for Reordering

```rust
// In src/routes/pet_profile_images.rs

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ImageOrderUpdate {
    pub image_id: i32,
    pub sort_order: i32,
}

#[derive(Deserialize)]
pub struct BulkImageOrder {
    pub images: Vec<ImageOrderUpdate>,
}

// PUT /api/pet-profiles/{id}/images/order
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn update_image_order(
    app_data: web::Data<AppData>,
    profile_id: web::Path<PetProfileId>,
    order_data: web::Json<BulkImageOrder>,
    auth: CookieAuth,
) -> Result<HttpResponse, DomainError> {
    let user_id = get_user_id_from_auth(&auth, &app_data).await?;

    // Verify ownership
    let profile = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profiles::get_pet_profile(profile_id.0, &mut conn)
    })
    .await??;

    if profile.user_id != user_id {
        return Err(DomainError::new_forbidden_error(
            "You can only reorder images for your own pet profiles".to_string()
        ));
    }

    web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::pet_profile_images::update_image_order(
            &profile_id.0,
            &order_data.images.iter().map(|i| (i.image_id, i.sort_order)).collect::<Vec<_>>(),
            &mut conn
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json("Order updated successfully"))
}
```

### Frontend Implementation Example (React)

```tsx
const ImageGallery = ({ images, profileId }) => {
  const [draggedIndex, setDraggedIndex] = useState(null);

  const handleDragStart = (index: number) => {
    setDraggedIndex(index);
  };

  const handleDragOver = (index: number) => {
    if (draggedIndex === null || draggedIndex === index) return;

    const newImages = [...images];
    const [draggedItem] = newImages.splice(draggedIndex, 1);
    newImages.splice(index, 0, draggedItem);

    // Update order in DB
    updateImageOrder(
      profileId,
      newImages.map((img, i) => ({
        image_id: img.id,
        sort_order: i,
      })),
    );

    setImages(newImages);
    setDraggedIndex(index);
  };

  return (
    <div className="gallery">
      {images.map((img, index) => (
        <div
          key={img.id}
          draggable
          onDragStart={() => handleDragStart(index)}
          onDragOver={() => handleDragOver(index)}
        >
          <img src={img.image_url} alt="Profile" />
        </div>
      ))}
    </div>
  );
};
```

## Authorization Strategy

### Route-Level Ownership Enforcement

Authorization should be handled at the API route level rather than database action level:

1. **Middlewares handle authentication** using existing [`cookie_auth`](src/utils/cookie_auth.rs:72)
2. **Routes enforce ownership** before calling database actions
3. **Database actions remain generic** and reusable
4. **Admin endpoints can bypass ownership** when needed

### Sample JSON Payloads

```json
// Creating a pet profile
{
  "user_id": 123,
  "pet_name": "Fluffy",
  "pet_type": "cat",
  "breed": "Persian",
  "age": 3,
  "weight_kg": 4.5,
  "gender": "female",
  "size": "medium",
  "bio": "A gentle and affectionate Persian cat who loves cuddles.",
  "personality_traits": ["calm", "affectionate", "playful"],
  "good_with_cats": true,
  "good_with_dogs": false,
  "good_with_kids": true,
  "house_trained": true,
  "vaccinated": true,
  "spayed_neutered": true
}

// Updating a pet profile
{ "id": 1,
  "bio": "Updated bio with more details",
  "likes": ["catnip", "laser pointers", "sunny spots"]
}
```

## Implementation Steps

1. **Update migration** add multi-pet features
2. **Create [`PetProfile` models](src/models/pet_profiles.rs:1)** with enums and validation
3. **Implement CRUD actions** in [`src/actions/pet_profiles.rs`](src/actions/pet_profiles.rs:1)
4. **Add API routes** in [`src/routes/pet_profiles.rs`](src/routes/pet_profiles.rs:1)
5. **Register routes** in [`src/main.rs`](src/main.rs:1)
6. **Update schema** registration in [`src/models.rs`](src/models.rs:1)
7. **Add integration tests** in [`tests/integration/pet_profiles.rs`](tests/integration/pet_profiles.rs:1)

## Dependencies

No additional dependencies required beyond existing Diesel and Actix setup.

## Testing Strategy

- Unit tests for model validation
- Integration tests for API endpoints
- Database transaction tests for multi-pet scenarios
- Ownership verification tests

This implementation provides a robust foundation for a pet dating app with support for multiple pets per user, proper ownership validation, and comprehensive CRUD operations.
