CREATE TABLE messages (
    id                      UUID        PRIMARY KEY,
    app_id                  UUID        NOT NULL REFERENCES applications(id),
    event_type_id           UUID        NOT NULL REFERENCES event_types(id),
    payload                 JSONB       NOT NULL,
    idempotency_key         TEXT        NOT NULL,
    idempotency_expires_at  TIMESTAMPTZ NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL
);
CREATE UNIQUE INDEX idx_messages_app_idempotency ON messages(app_id, idempotency_key);
