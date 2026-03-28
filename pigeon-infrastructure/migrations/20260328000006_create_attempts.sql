CREATE TABLE attempts (
    id              UUID        PRIMARY KEY,
    message_id      UUID        NOT NULL REFERENCES messages(id),
    endpoint_id     UUID        NOT NULL REFERENCES endpoints(id),
    status          TEXT        NOT NULL DEFAULT 'pending',
    response_code   SMALLINT,
    response_body   TEXT,
    attempted_at    TIMESTAMPTZ,
    next_attempt_at TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_attempts_next ON attempts(next_attempt_at) WHERE status = 'pending';
