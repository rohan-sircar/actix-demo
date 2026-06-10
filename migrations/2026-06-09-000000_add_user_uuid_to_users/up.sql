ALTER TABLE users ADD COLUMN user_uuid UUID NOT NULL DEFAULT gen_random_uuid();
CREATE UNIQUE INDEX users_user_uuid_idx ON users(user_uuid);
UPDATE users SET user_uuid = gen_random_uuid() WHERE user_uuid IS NULL;
