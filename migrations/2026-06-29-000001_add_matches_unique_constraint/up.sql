ALTER TABLE matches ADD CONSTRAINT chk_matches_like_order CHECK (like_id_a < like_id_b);
