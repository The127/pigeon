CREATE TABLE audit_log (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    command_name    TEXT        NOT NULL,
    actor           TEXT        NOT NULL,
    org_id          UUID        NOT NULL,
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT now(),
    success         BOOLEAN     NOT NULL,
    error_message   TEXT
);

CREATE INDEX idx_audit_log_org_timestamp ON audit_log(org_id, timestamp DESC);
CREATE INDEX idx_audit_log_actor ON audit_log(actor, timestamp DESC);
