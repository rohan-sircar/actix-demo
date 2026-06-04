DO $$ BEGIN
    CREATE TYPE oauth_provider_type AS ENUM ('github', 'google');
EXCEPTION WHEN duplicate_object THEN null;
END $$;

ALTER TABLE users
ADD COLUMN oauth_provider oauth_provider_type DEFAULT NULL,
ADD COLUMN oauth_uid VARCHAR(128) DEFAULT NULL;

CREATE UNIQUE INDEX idx_users_oauth_provider_uid ON users (oauth_provider, oauth_uid)
WHERE oauth_provider IS NOT NULL;
