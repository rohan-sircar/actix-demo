DROP INDEX IF EXISTS users_user_uuid_idx;
ALTER TABLE users DROP COLUMN IF EXISTS user_uuid;
