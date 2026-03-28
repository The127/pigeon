CREATE TABLE event_types (
    id         UUID        PRIMARY KEY,
    app_id     UUID        NOT NULL REFERENCES applications(id),
    name       TEXT        NOT NULL,
    schema     JSONB,
    created_at TIMESTAMPTZ NOT NULL
);
CREATE UNIQUE INDEX idx_event_types_app_name ON event_types(app_id, name);
