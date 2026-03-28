# Pigeon — Self-hosted Webhook Delivery Service

## Architecture
Hexagonal / ports-and-adapters with CQRS mediator dispatch.
Compiler-enforced boundaries via Cargo workspace dependency graph.
Multi-tenant: Organization is the tenant boundary, all data scoped via org_id.

### Crate dependency rules (violations = compiler error)
- **pigeon-macros** — proc-macro crate, no pigeon deps
- **pigeon-domain** — pure types; depends on pigeon-macros only
- **pigeon-application** — commands, queries, ports (traits), mediator; depends on domain only
- **pigeon-infrastructure** — adapters implementing ports; depends on application + domain
- **pigeon-api** — axum handlers, auth middleware, thin dispatch; depends on application + domain
- **pigeon-server** — composition root + CLI; depends on all crates (only one allowed to)

### Data model
```
Organization (tenant)
  └── OidcConfig (issuer_url + audience → org identity)
  └── Applications
        ├── EventTypes
        ├── Endpoints (subscribed to EventTypes via endpoint_events)
        └── Messages → Attempts → DeadLetters
```

### Key patterns
- **Unit of Work (change tracker)** — EF Core-style: store methods record changes in memory, `commit()` opens a short Postgres transaction and flushes all changes. No DB transaction held during handler execution.
- **Mediator pipeline** — `Command`/`Query` traits, `CommandHandler`/`QueryHandler`, `PipelineBehavior` middleware
- **Pipeline behaviors** — `TransactionBehavior` (begin/commit/rollback), `AuditBehavior` (audit log on success)
- **Optimistic concurrency** — `Version` type mapped to Postgres `xmin` system column. `UPDATE ... WHERE xmin::text::bigint = $version`, rows_affected=0 → Conflict
- **Reconstitution** — `#[derive(Reconstitute)]` generates `{Name}State` struct + `reconstitute()` method
- **Idempotency** — `IdempotencyKey` newtype on messages, `insert_or_get_existing` with `ON CONFLICT DO NOTHING`
- **Dead letters** — first-class `DeadLetter` entity after retry exhaustion
- **Delivery worker** — `DeliveryWorkerService` polls `DeliveryQueue` port, delivers via `WebhookHttpClient` port. Uses direct SQL (not UoW) — `SELECT ... FOR UPDATE SKIP LOCKED` dequeue, HMAC-SHA256 signing (`X-Pigeon-Signature`), exponential backoff retry, dead lettering. Runs as `pigeon worker` or alongside API via `pigeon serve`
- **Domain events** — transactional outbox: events INSERT'd into `event_outbox` table during UoW commit (same tx), outbox worker polls and processes. First event: `DeadLettered`

### Multitenancy
- **Tenant = Organization**, identified by OIDC (issuer_url, audience) pair
- **Auth flow:** JWT Bearer → decode iss+aud → lookup OidcConfig → validate signature via cached JWKS (1h TTL, force-refresh on key miss) → inject AuthContext(org_id, user_id)
- **Tenant scoping:** org_id from JWT (never from URL), cross-tenant access returns NotFound (not Forbidden)
- **SQL-level org enforcement:** all child entity queries JOIN through `applications WHERE org_id = $org_id`. No load-then-check pattern — the SQL itself enforces tenant isolation. If the app doesn't belong to the org, the query returns no rows.
- **Bootstrap org:** `PIGEON_BOOTSTRAP_ORG_ENABLED=true` creates a "system" org on startup if none exist (idempotent)
- **Admin API:** `/admin/v1/` for org + OIDC config management, protected by JWT auth + bootstrap org restriction
- **Tenant API:** `/api/v1/` for application data, requires JWT auth
- **Observability:** `metrics` facade for Prometheus counters/histograms/gauges, `GET /metrics` endpoint, `x-request-id` correlation header + tracing spans per request and per delivery attempt

### Visibility
`pub(crate)` / `pub(super)` by default. `pub` only at crate boundaries where needed.

## Testing strategy
- **BDD**: cucumber-rs in `pigeon-application/tests/` with `.feature` files in `tests/features/`
- **Mocks**: mockall `#[automock]` on port traits (gated behind `test-support` feature) — default for port mocking
- **Fakes**: hand-written fakes in `test_support/fakes.rs` only when stateful behavior is needed (e.g., `FakeUnitOfWork` — document why)
- **Fake data**: `fake-rs` via `XYZState::fake()` builders and `any_*()` factory functions in `pigeon-domain/src/test_support.rs`
- **HTTP fakes**: mockito for external HTTP endpoints (dev-dep in pigeon-infrastructure)
- **Integration**: testcontainers for Postgres adapter tests
- **Proc-macro**: trybuild for compile-pass/compile-fail tests
- **API handler tests**: tower::ServiceExt::oneshot with hand-written fake handlers
- Tests first or alongside, never after

### Feature flags
- `pigeon-domain/test-support` — enables `fake-rs`, `test_support` module with `::fake()` builders and `any_*()` factories
- `pigeon-application/test-support` — enables domain test-support + `mockall` automocks on port traits + `test_support` module with fakes

## API routes

### Tenant API (JWT auth required)
- `POST /api/v1/applications` — create application
- `GET /api/v1/applications` — list applications (scoped to org)
- `GET /api/v1/applications/{id}` — get application
- `PUT /api/v1/applications/{id}` — update application
- `DELETE /api/v1/applications/{id}` — delete application
- `POST /api/v1/applications/{app_id}/messages` — send message (idempotent fan-out)
- `POST /api/v1/applications/{app_id}/event-types` — create event type
- `GET /api/v1/applications/{app_id}/event-types` — list event types
- `GET /api/v1/applications/{app_id}/event-types/{id}` — get event type
- `PUT /api/v1/applications/{app_id}/event-types/{id}` — update event type
- `DELETE /api/v1/applications/{app_id}/event-types/{id}` — delete event type
- `POST /api/v1/applications/{app_id}/endpoints` — create endpoint
- `GET /api/v1/applications/{app_id}/endpoints` — list endpoints
- `GET /api/v1/applications/{app_id}/endpoints/{id}` — get endpoint
- `PUT /api/v1/applications/{app_id}/endpoints/{id}` — update endpoint
- `DELETE /api/v1/applications/{app_id}/endpoints/{id}` — delete endpoint
- `POST /api/v1/applications/{app_id}/dead-letters/{id}/replay` — replay dead letter
- `POST /api/v1/applications/{app_id}/attempts/{id}/retry` — retry failed attempt
- `POST /api/v1/applications/{app_id}/endpoints/{id}/test` — send test event

### Admin API (JWT auth, bootstrap org only)
- `POST /admin/v1/organizations` — create organization
- `GET /admin/v1/organizations` — list organizations
- `GET /admin/v1/organizations/{id}` — get organization
- `PUT /admin/v1/organizations/{id}` — update organization
- `DELETE /admin/v1/organizations/{id}` — delete organization
- `POST /admin/v1/organizations/{org_id}/oidc-configs` — create OIDC config
- `GET /admin/v1/organizations/{org_id}/oidc-configs` — list OIDC configs
- `GET /admin/v1/organizations/{org_id}/oidc-configs/{id}` — get OIDC config
- `DELETE /admin/v1/organizations/{org_id}/oidc-configs/{id}` — delete OIDC config

### Health (no auth)
- `GET /health` — liveness (always 200)
- `GET /health/ready` — readiness (200 if DB reachable, 503 if not)
- `GET /api/openapi.json` — OpenAPI spec

## Build & run (justfile)
```sh
# Build & check
just check            # type-check workspace
just build            # debug build
just build-release    # release build
just test             # all tests
just clippy           # lint (warnings = errors)
just fmt / fmt-check  # format
just lint             # clippy + fmt-check
just ci               # fmt-check + clippy + test

# Development
just dev-up           # start local Postgres (docker compose)
just dev-down         # stop Postgres
just dev-reset        # stop + delete data
just dev-migrate      # run migrations
just dev-run          # start API server
just dev-watch        # watch mode (cargo-watch)
```

## Configuration (env vars)
- `DATABASE_URL` — Postgres connection string (required)
- `PIGEON_LISTEN_ADDR` — API listen address (default `0.0.0.0:3000`)
- `PIGEON_BOOTSTRAP_ORG_ENABLED` — create system org on startup (default `false`)
- `PIGEON_BOOTSTRAP_ORG_NAME` — bootstrap org name (default `System`)
- `PIGEON_BOOTSTRAP_ORG_SLUG` — bootstrap org slug (default `system`)
- `PIGEON_JWKS_CACHE_TTL_SECS` — JWKS cache TTL in seconds (default `3600`)
- `PIGEON_WORKER_BATCH_SIZE` — attempts to dequeue per poll (default `10`)
- `PIGEON_WORKER_POLL_INTERVAL_MS` — ms between polls when idle (default `1000`)
- `PIGEON_WORKER_MAX_RETRIES` — max delivery attempts before dead letter (default `5`)
- `PIGEON_WORKER_BACKOFF_BASE_SECS` — exponential backoff base (default `30`)
- `PIGEON_WORKER_MAX_BACKOFF_SECS` — backoff cap (default `3600`)
- `PIGEON_WORKER_HTTP_TIMEOUT_SECS` — webhook HTTP request timeout (default `30`)
- `PIGEON_WORKER_CLEANUP_INTERVAL_SECS` — idempotency key cleanup interval (default `3600`)
- `PIGEON_WORKER_AUTO_DISABLE_THRESHOLD` — consecutive dead letters before auto-disabling endpoint (default `5`, `0` to disable)

## Conventions
- Manual constructor injection, composition root in pigeon-server
- One feature at a time
- Do not proceed to next step without explicit user approval
- Never generate full codebase speculatively

## Definition of Done
Every change must satisfy before commit:
- Does this have unit tests?
- Does this have BDD scenarios if it touches domain behavior?
- Does clippy pass?
