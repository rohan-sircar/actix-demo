CREATE TABLE matches (
    id SERIAL PRIMARY KEY,
    like_id_a INTEGER NOT NULL REFERENCES likes(id),
    like_id_b INTEGER NOT NULL REFERENCES likes(id),
    matched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT like_id_a_less_than_like_id_b CHECK (like_id_a < like_id_b)
);

ALTER TABLE likes DROP COLUMN if exists is_match;
ALTER TABLE likes DROP COLUMN if exists matched_at;
ALTER TABLE likes DROP COLUMN if exists reciprocal_like_id;
