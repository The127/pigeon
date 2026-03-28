CREATE TABLE oidc_configs (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id),
    issuer_url  TEXT        NOT NULL,
    audience    TEXT        NOT NULL,
    jwks_url    TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL
);
CREATE UNIQUE INDEX idx_oidc_configs_issuer_audience ON oidc_configs(issuer_url, audience);
