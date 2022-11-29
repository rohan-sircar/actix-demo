-- Your SQL goes here
CREATE TYPE role_name AS ENUM ('role_super_user', 'role_admin', 'role_user');

CREATE TABLE if not exists roles (
    id SERIAL PRIMARY KEY NOT NULL,
    role_name role_name NOT NULL UNIQUE
);

INSERT INTO
    roles (role_name)
VALUES
    ('role_super_user');

INSERT INTO
    roles (role_name)
VALUES
    ('role_admin');

INSERT INTO
    roles (role_name)
VALUES
    ('role_user');

CREATE TABLE IF NOT EXISTS users_roles (
    id SERIAL PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    CONSTRAINT fk_users_roles_user_id FOREIGN KEY(user_id) REFERENCES users(id),
    CONSTRAINT fk_users_roles_role_id FOREIGN KEY(role_id) REFERENCES roles(id)
);

