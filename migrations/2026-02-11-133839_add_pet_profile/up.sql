-- Create pet_types enum
CREATE TYPE pet_type AS ENUM ('dog', 'cat');

CREATE TYPE gender_type AS ENUM ('male', 'female');

CREATE TYPE size_type AS ENUM ('toy', 'small', 'medium', 'large', 'giant');

CREATE TYPE coat_type AS ENUM (
    'short',
    'medium',
    'long',
    'curly',
    'wire',
    'hairless'
);

CREATE TYPE energy_level_type AS ENUM ('low', 'medium', 'high', 'extreme');

CREATE TYPE trainability_type AS ENUM ('easy', 'moderate', 'challenging');

CREATE TYPE barking_level_type AS ENUM ('quiet', 'moderate', 'loud');

CREATE TYPE adoption_status_type AS ENUM ('adoptable', 'foster', 'available');

-- Create pet_basic_info table
CREATE TABLE pet_basic_info (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Core pet info
    pet_name VARCHAR(100) NOT NULL,
    pet_type pet_type NOT NULL,
    breed VARCHAR(100) NOT NULL,
    age INTEGER NOT NULL CHECK (
        age >= 0
        AND age <= 30
    ),
    weight_kg FLOAT4 NOT NULL CHECK (weight_kg > 0),
    gender gender_type NOT NULL,
    size size_type,
    color VARCHAR(50),
    coat_type coat_type,
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on uuid for fast lookups
CREATE INDEX idx_pet_basic_info_uuid ON pet_basic_info(uuid);

-- Create pet_personality_traits table
CREATE TABLE pet_personality_traits (
    id SERIAL PRIMARY KEY,
    pet_profile_uuid UUID NOT NULL REFERENCES pet_basic_info(uuid) ON DELETE CASCADE,
    -- Personality
    bio TEXT,
    personality_traits TEXT [] DEFAULT '{}',
    good_with_dogs BOOLEAN,
    good_with_cats BOOLEAN,
    good_with_kids BOOLEAN,
    house_trained BOOLEAN,
    vaccinated BOOLEAN,
    spayed_neutered BOOLEAN,
    microchipped BOOLEAN
);

-- Create pet_activities table
CREATE TABLE pet_activities (
    id SERIAL PRIMARY KEY,
    pet_profile_uuid UUID NOT NULL REFERENCES pet_basic_info(uuid) ON DELETE CASCADE,
    -- Activities
    favorite_activities TEXT [] DEFAULT '{}',
    likes TEXT [] DEFAULT '{}',
    dislikes TEXT [] DEFAULT '{}',
    energy_level energy_level_type,
    trainability trainability_type,
    barking_level barking_level_type
);

-- Create pet_location_owner table
CREATE TABLE pet_location_owner (
    id SERIAL PRIMARY KEY,
    pet_profile_uuid UUID NOT NULL REFERENCES pet_basic_info(uuid) ON DELETE CASCADE,
    -- Owner info
    owner_name VARCHAR(100) NOT NULL,
    location VARCHAR(100) NOT NULL,
    address TEXT,
    lat DECIMAL(10, 8),
    lng DECIMAL(11, 8)
);

-- Create pet_adoption_details table
CREATE TABLE pet_adoption_details (
    id SERIAL PRIMARY KEY,
    pet_profile_uuid UUID NOT NULL REFERENCES pet_basic_info(uuid) ON DELETE CASCADE,
    -- Special considerations
    special_needs BOOLEAN DEFAULT false,
    special_needs_description TEXT,
    adoption_status adoption_status_type,
    shelter_name VARCHAR(100)
);

-- Create indexes on foreign keys
CREATE INDEX idx_pet_basic_info_user_id ON pet_basic_info(user_id);

CREATE INDEX idx_pet_personality_traits_pet_uuid ON pet_personality_traits(pet_profile_uuid);

CREATE INDEX idx_pet_activities_pet_uuid ON pet_activities(pet_profile_uuid);

CREATE INDEX idx_pet_location_owner_pet_uuid ON pet_location_owner(pet_profile_uuid);

CREATE INDEX idx_pet_adoption_details_pet_uuid ON pet_adoption_details(pet_profile_uuid);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER update_pet_basic_info_updated_at
    BEFORE UPDATE
    ON pet_basic_info
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create separate image table
CREATE TABLE pet_profile_images (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    pet_profile_uuid UUID NOT NULL REFERENCES pet_basic_info(uuid) ON DELETE CASCADE,
    image_url TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT false,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_pet_profile_images_uuid ON pet_profile_images(uuid);

CREATE INDEX idx_pet_profile_images_pet_profile_uuid ON pet_profile_images(pet_profile_uuid);

CREATE UNIQUE INDEX idx_pet_profile_images_primary ON pet_profile_images(pet_profile_uuid)
WHERE
    is_primary = true;