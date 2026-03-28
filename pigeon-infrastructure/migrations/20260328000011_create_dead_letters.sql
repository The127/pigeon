CREATE TABLE dead_letters (
    id                  UUID        PRIMARY KEY,
    message_id          UUID        NOT NULL REFERENCES messages(id),
    endpoint_id         UUID        NOT NULL REFERENCES endpoints(id),
    app_id              UUID        NOT NULL REFERENCES applications(id),
    last_response_code  SMALLINT,
    last_response_body  TEXT,
    dead_lettered_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    replayed_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_dead_letters_app_id ON dead_letters(app_id);
CREATE INDEX idx_dead_letters_endpoint_id ON dead_letters(endpoint_id);
