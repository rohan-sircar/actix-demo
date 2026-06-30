CREATE TYPE like_direction AS ENUM ('like', 'dislike');

CREATE TABLE likes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pet_owner_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pet_id INTEGER NOT NULL REFERENCES pets(id) ON DELETE CASCADE,
    direction like_direction NOT NULL,
    is_match BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_likes_user_pet_owner ON likes(user_id, pet_owner_id);
CREATE UNIQUE INDEX idx_likes_user_pet_unique ON likes(pet_id, user_id);
CREATE INDEX idx_likes_is_match ON likes(is_match);
