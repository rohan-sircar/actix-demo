ALTER TABLE users ADD COLUMN email VARCHAR(255) UNIQUE NOT NULL DEFAULT 'noreply@actix-demo.local';
CREATE INDEX idx_users_email ON users(email);
