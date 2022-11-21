-- This file should undo anything in `up.sql`
CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO
    users (id, name, password, created_at)
SELECT
    id,
    name,
    password,
    created_at
FROM
    users2;

DROP TABLE users2;

DROP TABLE roles;