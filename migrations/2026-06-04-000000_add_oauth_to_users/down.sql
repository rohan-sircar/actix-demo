DROP INDEX IF EXISTS idx_users_oauth_provider_uid;

ALTER TABLE users
DROP COLUMN IF EXISTS oauth_provider,
DROP COLUMN IF EXISTS oauth_uid;

DROP TYPE IF EXISTS oauth_provider_type;
