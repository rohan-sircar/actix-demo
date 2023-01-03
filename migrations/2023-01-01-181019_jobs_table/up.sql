-- Your SQL goes here
CREATE TYPE job_status AS ENUM ('pending', 'completed', 'aborted', 'failed');

CREATE TABLE IF NOT EXISTS jobs (
    id SERIAL PRIMARY KEY NOT NULL,
    job_id UUID NOT NULL UNIQUE,
    started_by INTEGER NOT NULL,
    status job_status NOT NULL,
    status_message VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_jobs_started_by FOREIGN KEY(started_by) REFERENCES users(id)
);