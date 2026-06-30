CREATE TABLE IF NOT EXISTS matches (
    id SERIAL PRIMARY KEY,
    like_id_a INTEGER NOT NULL REFERENCES likes(id) ON DELETE CASCADE,
    like_id_b INTEGER NOT NULL REFERENCES likes(id) ON DELETE CASCADE,
    matched_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_matches_like_pair ON matches(like_id_a, like_id_b);
