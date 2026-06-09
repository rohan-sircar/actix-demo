CREATE TYPE pet_gender AS ENUM ('male', 'female', 'unspecified');

CREATE TABLE pets (
    id SERIAL PRIMARY KEY,
    pet_uuid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    species VARCHAR(50) NOT NULL,
    breed VARCHAR(200),
    date_of_birth DATE,
    gender pet_gender,
    weight DOUBLE PRECISION,
    color_markings VARCHAR(200),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pets_pet_uuid ON pets(pet_uuid);

CREATE OR REPLACE FUNCTION update_pets_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_pets_updated_at
    BEFORE UPDATE ON pets
    FOR EACH ROW EXECUTE FUNCTION update_pets_updated_at();

CREATE TABLE personality_traits (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO personality_traits (name) VALUES
    ('playful'), ('calm'), ('energetic'), ('affectionate'),
    ('independent'), ('curious'), ('loyal'), ('gentle'),
    ('stubborn'), ('social'), ('shy'), ('protective'),
    ('vocally expressive'), ('food motivated'), ('smart');

CREATE TABLE pet_personality_traits (
    pet_id INTEGER NOT NULL REFERENCES pets(id) ON DELETE CASCADE,
    trait_id INTEGER NOT NULL REFERENCES personality_traits(id) ON DELETE CASCADE,
    PRIMARY KEY (pet_id, trait_id)
);
