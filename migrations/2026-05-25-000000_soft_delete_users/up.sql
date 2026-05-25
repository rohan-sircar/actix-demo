-- Add soft-delete column to users
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMP;

-- Make jobs.started_by nullable so we can orphan jobs on user deletion
ALTER TABLE jobs ALTER COLUMN started_by DROP NOT NULL;
