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

ALTER TABLE
    USERS
ADD
    role_id INTEGER NOT NULL DEFAULT 3;

ALTER TABLE
    USERS
ADD
    FOREIGN KEY (role_id) REFERENCES roles(id);

-- CREATE TABLE users2 (
--     id INTEGER PRIMARY KEY NOT NULL,
--     name VARCHAR NOT NULL,
--     password VARCHAR NOT NULL,
--     role_id INTEGER NOT NULL DEFAULT 3 REFERENCES roles(id),
--     created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
-- );
-- INSERT INTO
--     users2 (id, name, password, created_at)
-- SELECT
--     *
-- FROM
--     users;
-- DROP TABLE users;