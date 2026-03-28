CREATE TABLE endpoints (
    id              UUID        PRIMARY KEY,
    app_id          UUID        NOT NULL REFERENCES applications(id),
    url             TEXT        NOT NULL,
    signing_secret  TEXT        NOT NULL,
    enabled         BOOLEAN     NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL
);
