-- Rollback: restore NOT NULL constraint first (requires no NULL values)
-- In practice, this migration is irreversible without data cleanup.
-- For safety, we just drop the column.
ALTER TABLE users DROP COLUMN deleted_at;
ALTER TABLE jobs ALTER COLUMN started_by SET NOT NULL;
