-- Your SQL goes here
CREATE TABLE users (
  id INTEGER PRIMARY KEY NOT NULL ,
  name VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
)
