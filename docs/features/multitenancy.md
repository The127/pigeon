# Multitenancy

## Tenant model

Each **Organization** is a tenant. Organizations are identified by their OIDC configuration (issuer URL + audience pair).

## Authentication flow

1. Client sends JWT Bearer token in the `Authorization` header
2. Pigeon decodes the JWT (unvalidated) to extract `iss` and `aud`
3. Looks up the matching `OidcConfig` by issuer + audience
4. Fetches the JWKS from the config's `jwks_url` (cached, 1h TTL)
5. Validates the JWT signature, expiry, issuer, and audience
6. Injects `AuthContext(org_id, user_id)` into the request

## Tenant isolation

- `org_id` comes from the JWT, never from the URL
- All SQL queries JOIN through `applications WHERE org_id = $org_id`
- Cross-tenant access returns 404 (not 403) — no information leakage
- No load-then-check pattern — the SQL itself enforces isolation

## Admin vs tenant API

- `/api/v1/` — tenant API, scoped to the authenticated org
- `/admin/v1/` — admin API, restricted to the bootstrap organization

Admin users can manage all organizations and their OIDC configs. Regular users can manage their own org's OIDC config via `/api/v1/oidc-configs`.

## Bootstrap organization

On first startup with `PIGEON_BOOTSTRAP_ORG_ENABLED=true`, Pigeon creates a "System" organization with the provided OIDC config. This org has admin privileges over the `/admin/v1/` API.
