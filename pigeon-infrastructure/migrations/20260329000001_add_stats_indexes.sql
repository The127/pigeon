-- Index for time-bucketed stats queries on completed attempts
CREATE INDEX idx_attempts_app_attempted_at ON attempts(
    (SELECT app_id FROM messages WHERE messages.id = attempts.message_id),
    attempted_at
) WHERE attempted_at IS NOT NULL;

-- Simpler approach: denormalize app_id onto attempts for efficient stats queries.
-- The join through messages is expensive for aggregation.
-- Instead, index the message table and use it for joins.
DROP INDEX IF EXISTS idx_attempts_app_attempted_at;

CREATE INDEX idx_attempts_status_attempted ON attempts(status, attempted_at)
WHERE attempted_at IS NOT NULL;

CREATE INDEX idx_messages_app_created ON messages(app_id, created_at);
