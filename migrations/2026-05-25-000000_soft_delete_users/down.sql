-- Rollback: drop soft-delete column and restore NOT NULL on jobs.started_by
-- NOTE: This migration is effectively irreversible if jobs were orphaned
-- (started_by set to NULL). The DELETE below removes orphaned rows to
-- satisfy the NOT NULL constraint, which destroys job history.
ALTER TABLE users DROP COLUMN deleted_at;
DELETE FROM jobs WHERE started_by IS NULL;
ALTER TABLE jobs ALTER COLUMN started_by SET NOT NULL;
