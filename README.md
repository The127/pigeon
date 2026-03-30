<p align="center">
  <img src="pigeon.svg" alt="Pigeon" width="64" height="64">
</p>

<h1 align="center">Pigeon</h1>

<p align="center">Self-hosted webhook delivery service. Send events, Pigeon delivers them.</p>

<p align="center">
  <a href="https://the127.github.io/pigeon/">Documentation</a> ·
  <a href="https://github.com/The127/pigeon/releases">Releases</a>
</p>

---

## What it does

You send a message with an event type and a JSON payload. Pigeon fans it out to all endpoints subscribed to that event type, signs each delivery with HMAC-SHA256, retries on failure with exponential backoff, and dead-letters after exhaustion. Everything is scoped per organization via OIDC-based multitenancy.

## Quick start

```bash
docker pull ghcr.io/the127/pigeon-server:1
docker pull ghcr.io/the127/pigeon-ui:1

# Minimal: API + worker + Postgres
docker run -d --name pigeon \
  -e DATABASE_URL=postgres://user:pass@host/pigeon \
  -e PIGEON_BOOTSTRAP_ORG_ENABLED=true \
  -e PIGEON_BOOTSTRAP_OIDC_ISSUER_URL=https://your-oidc-provider \
  -e PIGEON_BOOTSTRAP_OIDC_AUDIENCE=pigeon-api \
  -p 3000:3000 \
  ghcr.io/the127/pigeon-server:1
```

The server runs migrations on startup. No separate step needed.

## Architecture

Hexagonal / ports-and-adapters. Rust workspace with compiler-enforced crate boundaries.

```
pigeon-macros        proc macros (Reconstitute derive)
pigeon-domain        aggregates, value objects, domain events
pigeon-application   commands, queries, ports, mediator pipeline
pigeon-infrastructure   Postgres adapters, HTTP webhook client
pigeon-api           axum handlers, JWT auth, OpenAPI
pigeon-server        composition root, CLI
pigeon-ui            Vue 3 + TypeScript frontend
```

### Data model

```
Organization (tenant)
  └── OidcConfig
  └── Applications
        ├── EventTypes
        ├── Endpoints → signing_secrets (Pigeon-generated, rotatable)
        └── Messages → Attempts → DeadLetters
```

## API

All endpoints require JWT Bearer auth. org_id is resolved from the token, never from the URL.

### Tenant API (`/api/v1/`)

| Resource | Endpoints |
|----------|-----------|
| Applications | CRUD + stats |
| Event Types | CRUD + stats |
| Endpoints | CRUD + stats + rotate/revoke signing secrets + test event |
| Messages | Send (idempotent) + list + retrigger |
| Attempts | List by message + retry failed |
| Dead Letters | List + replay |
| OIDC Configs | List + create + delete (own org) |
| Audit Log | List with filters |

### Admin API (`/admin/v1/`)

Organization and OIDC config management. Requires bootstrap org JWT.

### Other

- `GET /api/openapi.json` — OpenAPI 3.0 spec
- `GET /health` / `GET /health/ready` — liveness / readiness
- `GET /metrics` — Prometheus metrics

## Delivery

- **Fan-out**: message → one attempt per subscribed endpoint
- **Signing**: HMAC-SHA256 with all active signing secrets. Header: `X-Pigeon-Signature: sha256=<sig1>,sha256=<sig2>`
- **Retry**: exponential backoff (configurable base/max), `SELECT ... FOR UPDATE SKIP LOCKED`
- **Dead letter**: after max retries, attempts are dead-lettered. Replayable via API.
- **Auto-disable**: endpoints with consecutive failures above threshold are automatically disabled
- **Idempotency**: messages keyed by `idempotency_key`, duplicates return existing message

## Signing secret rotation

Pigeon generates signing secrets (`whsec_` + 64 hex chars). Endpoints support up to 2 active secrets for zero-downtime rotation:

1. `POST .../endpoints/{id}/rotate` — generates new secret, keeps old
2. Pigeon signs deliveries with **all** active secrets
3. Update your consumer to verify with the new secret
4. `DELETE .../endpoints/{id}/secrets/1` — revoke the old secret

Full secrets are shown only on create and rotate. Masked (`whsec_...abc123`) everywhere else.

## Configuration

All via environment variables. `PIGEON_` prefix for app config.

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | required | Postgres connection string |
| `PIGEON_LISTEN_ADDR` | `0.0.0.0:3000` | API listen address |
| `PIGEON_BOOTSTRAP_ORG_ENABLED` | `false` | Create bootstrap org on startup |
| `PIGEON_BOOTSTRAP_ORG_NAME` | `System` | Bootstrap org display name |
| `PIGEON_BOOTSTRAP_ORG_SLUG` | `system` | Bootstrap org URL slug |
| `PIGEON_BOOTSTRAP_OIDC_ISSUER_URL` | — | Required if bootstrap enabled |
| `PIGEON_BOOTSTRAP_OIDC_AUDIENCE` | — | Required if bootstrap enabled |
| `PIGEON_BOOTSTRAP_OIDC_JWKS_URL` | derived from issuer | JWKS endpoint override |
| `PIGEON_JWKS_CACHE_TTL_SECS` | `3600` | JWKS cache TTL |
| `PIGEON_WORKER_BATCH_SIZE` | `10` | Attempts dequeued per poll |
| `PIGEON_WORKER_POLL_INTERVAL_MS` | `1000` | Idle poll interval |
| `PIGEON_WORKER_MAX_RETRIES` | `5` | Attempts before dead letter |
| `PIGEON_WORKER_BACKOFF_BASE_SECS` | `30` | Exponential backoff base |
| `PIGEON_WORKER_MAX_BACKOFF_SECS` | `3600` | Backoff cap |
| `PIGEON_WORKER_HTTP_TIMEOUT_SECS` | `30` | Webhook request timeout |
| `PIGEON_WORKER_AUTO_DISABLE_THRESHOLD` | `5` | Consecutive dead letters before auto-disable (0 = off) |
| `PIGEON_WORKER_CLEANUP_INTERVAL_SECS` | `3600` | Idempotency key cleanup interval |

## Running modes

The server binary supports multiple subcommands:

```bash
pigeon serve    # API + worker (default in Docker)
pigeon api      # API only
pigeon worker   # Worker only
pigeon migrate  # Run migrations and exit
```

## Development

```bash
# Prerequisites: Rust, Node.js, Docker (for Postgres)

# Start Postgres
just dev-up

# Run migrations + start server
just dev-run

# Frontend
cd pigeon-ui && npm install && npm run dev

# Tests
just test          # all tests
just clippy        # lint
just ci            # fmt + clippy + test

# Generate OpenAPI client (requires running server)
cd pigeon-ui && npm run generate-api
```

## License

AGPL-3.0
