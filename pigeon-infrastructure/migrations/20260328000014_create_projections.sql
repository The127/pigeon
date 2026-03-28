CREATE TABLE endpoint_delivery_summary (
    endpoint_id          UUID    PRIMARY KEY REFERENCES endpoints(id),
    last_delivery_at     TIMESTAMPTZ,
    last_status          TEXT,
    total_success        BIGINT  NOT NULL DEFAULT 0,
    total_failure        BIGINT  NOT NULL DEFAULT 0,
    consecutive_failures BIGINT  NOT NULL DEFAULT 0
);

CREATE TABLE message_delivery_status (
    message_id      UUID    PRIMARY KEY REFERENCES messages(id),
    attempts_created INTEGER NOT NULL DEFAULT 0,
    succeeded        INTEGER NOT NULL DEFAULT 0,
    failed           INTEGER NOT NULL DEFAULT 0,
    dead_lettered    INTEGER NOT NULL DEFAULT 0
);
