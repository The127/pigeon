# Pigeon — Backlog

## Done

### ~~Delivery Worker~~
SKIP LOCKED dequeue from `attempts` table, HTTP POST to endpoint URL, exponential backoff retry, dead lettering after max retries exhausted.

### ~~HMAC Payload Signing~~
`X-Pigeon-Signature: sha256={hex}` via HMAC-SHA256 with endpoint signing_secret.

### ~~Attempt `duration_ms`~~
`duration_ms: Option<i64>` + `attempt_number: u32` on Attempt entity and migration.

## Priority: High

### Admin Route Authentication
`/admin/v1/*` routes currently have no auth — anyone can create/delete organizations and OIDC configs. Needs its own auth mechanism (bootstrap org's OIDC, API key, or separate admin token).

### Metrics / Prometheus Endpoint
Expose `GET /metrics` with Prometheus-format metrics. Key metrics:
- `pigeon_queue_depth` — pending attempts count
- `pigeon_delivery_total{status=success|failure|dead_letter}` — delivery counter
- `pigeon_delivery_duration_seconds` — histogram of HTTP delivery times
- `pigeon_message_total` — messages received counter

Hard to retrofit after the fact — add before the delivery worker goes to production.

### Correlation IDs
Assign `request_id` on API ingress, propagate `message_id` through fan-out and attempts. Add tracing spans so a single message can be traced end-to-end across fan-out to N endpoints with M attempts each. Use `tracing` spans + `x-request-id` header.

### Queue Depth Monitoring
Expose pending attempt count as a metric (ties into Prometheus endpoint). Alert when queue depth exceeds threshold — means workers are falling behind.

## Priority: Medium

### `Message.delivered_at`
Denormalized timestamp updated on last successful delivery for any endpoint. Avoids expensive `SELECT MAX(attempted_at) FROM attempts WHERE message_id = $1 AND status = 'succeeded'` queries. Add to domain entity + migration.

### Idempotency Key Cleanup Job
Worker task that periodically runs `DELETE FROM messages WHERE idempotency_expires_at < now()`. The `MessageStore.expire_idempotency_keys` port already exists — needs a scheduled invocation.

### ReplayDeadLetter Command
Re-enqueue a dead-lettered message for a specific endpoint. Creates new `Attempt` with status pending, resets attempt count, sets `replayed_at` on the `DeadLetter`. Domain entity and port already exist.

### Manual Retry of Specific Attempt
Different from replay — "retry this attempt now" before it's dead-lettered. Sets `next_attempt_at = now()` and `status = pending` on an existing failed attempt. Useful for operators investigating delivery issues.

### Endpoint Test Event
"Send a test event to this endpoint" — generates a synthetic message with a known payload and delivers immediately. Useful for onboarding and debugging. Similar to Svix's test event feature.

### Domain Event Dispatch
`AggregateRoot` trait and `DomainEvent` enum exist but nothing collects or dispatches events. Wire up event collection on entity mutations, dispatch after successful UoW commit. First use case: `DeadLettered` event.

### Integration Tests for Newer Features
Missing Postgres-level tests for:
- OidcConfig CRUD + unique constraint on (issuer_url, audience)
- Cross-tenant SQL isolation (org A can't query org B's data via JOIN)
- SendMessage full flow (message + attempts + idempotency against real Postgres)

### Configurable Backoff Cap
`PIGEON_WORKER_MAX_BACKOFF_SECS` env var exists (default 3600) but is not yet exposed through a runtime API or admin endpoint. Consider adding a `/admin/v1/worker/config` endpoint for dynamic tuning without restart.

### Signing Secret Rotation
No mechanism to rotate an endpoint's `signing_secret` without breaking in-flight deliveries. Design: dual-secret window — deliver signed with new secret, but during a configurable transition period include both old and new signatures so consumers can verify with either.

## Priority: Low

### Audit Log Postgres Implementation
`AuditStore` and `AuditBehavior` ports exist in the application layer. Needs a `PgAuditStore` adapter, `audit_log` migration, and wiring into the mediator pipeline.

### Config Crate
Currently raw `std::env::var` calls in `PigeonConfig`. Could use `config-rs` or `envy` for typed config with layered sources (env, file, defaults).

### Pagination Audit
Verify all list endpoints have `offset`/`limit` query params. Likely already complete but worth a sweep.

## Out of Scope (Noted)

### Inbound Webhook Signature Verification
Pigeon sends webhooks, it doesn't receive them from external services. If scope expands to receiving, inbound signatures need verification. Not currently planned.

### Frontend (pigeon-ui)
Vue 3 + TypeScript + shadcn-vue + generated OpenAPI client. Lives in `pigeon-ui/` directory. Blocked until API is stable and delivery worker is operational. OpenAPI spec already generated at `/api/openapi.json`.
