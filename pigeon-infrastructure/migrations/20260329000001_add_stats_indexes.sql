-- Indexes for time-bucketed stats queries

CREATE INDEX idx_attempts_status_attempted ON attempts(status, attempted_at)
WHERE attempted_at IS NOT NULL;

CREATE INDEX idx_messages_app_created ON messages(app_id, created_at);
