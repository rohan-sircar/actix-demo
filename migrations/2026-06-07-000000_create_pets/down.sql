DROP TABLE IF EXISTS pet_personality_traits;
DROP TABLE IF EXISTS personality_traits;
DROP TRIGGER IF EXISTS trg_pets_updated_at ON pets;
DROP FUNCTION IF EXISTS update_pets_updated_at();
DROP TABLE IF EXISTS pets;
