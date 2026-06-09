CREATE TABLE pet_images (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    pet_id INTEGER NOT NULL REFERENCES pets(id) ON DELETE CASCADE,
    thumbnail_key TEXT NOT NULL,
    medium_key TEXT NOT NULL,
    original_key TEXT NOT NULL,
    format VARCHAR(10) NOT NULL DEFAULT 'webp',
    is_primary BOOLEAN NOT NULL DEFAULT false,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pet_images_pet_id ON pet_images(pet_id);
