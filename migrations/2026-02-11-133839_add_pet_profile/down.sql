-- This file should undo anything in `up.sql`
-- Drop indexes
DROP INDEX IF EXISTS idx_pet_profile_images_uuid;
DROP INDEX IF EXISTS idx_pet_profile_images_primary;
DROP INDEX IF EXISTS idx_pet_profile_images_pet_profile_uuid;
DROP INDEX IF EXISTS idx_pet_adoption_details_pet_uuid;
DROP INDEX IF EXISTS idx_pet_location_owner_pet_uuid;
DROP INDEX IF EXISTS idx_pet_activities_pet_uuid;
DROP INDEX IF EXISTS idx_pet_personality_traits_pet_uuid;
DROP INDEX IF EXISTS idx_pet_basic_info_user_id;
DROP INDEX IF EXISTS idx_pet_basic_info_uuid;

-- Drop pet_profile_images table
DROP TABLE IF EXISTS pet_profile_images;

-- Drop pet_adoption_details table
DROP TABLE IF EXISTS pet_adoption_details;

-- Drop pet_location_owner table
DROP TABLE IF EXISTS pet_location_owner;

-- Drop pet_activities table
DROP TABLE IF EXISTS pet_activities;

-- Drop pet_personality_traits table
DROP TABLE IF EXISTS pet_personality_traits;

-- Drop trigger
DROP TRIGGER IF EXISTS update_pet_basic_info_updated_at ON pet_basic_info;

-- Drop trigger function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop pet_basic_info table
DROP TABLE IF EXISTS pet_basic_info;

-- Drop custom enums
DROP TYPE IF EXISTS adoption_status_type;

DROP TYPE IF EXISTS barking_level_type;

DROP TYPE IF EXISTS trainability_type;

DROP TYPE IF EXISTS energy_level_type;

DROP TYPE IF EXISTS coat_type;

DROP TYPE IF EXISTS size_type;

DROP TYPE IF EXISTS gender_type;

DROP TYPE IF EXISTS pet_type;