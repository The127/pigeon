CREATE TABLE applications (
    id         UUID        PRIMARY KEY,
    name       TEXT        NOT NULL,
    uid        TEXT        NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL
);
