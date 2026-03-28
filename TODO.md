# Pigeon — Backlog

## Done

### ~~Delivery Worker~~
SKIP LOCKED dequeue, HTTP POST, exponential backoff retry, dead lettering.

### ~~HMAC Payload Signing~~
`X-Pigeon-Signature: sha256={hex}` via HMAC-SHA256.

### ~~Attempt `duration_ms`~~
`duration_ms: Option<i64>` + `attempt_number: u32` on Attempt.

### ~~Metrics / Prometheus Endpoint~~
`GET /metrics` with `pigeon_messages_total`, `pigeon_delivery_total`, `pigeon_delivery_duration_seconds`, `pigeon_queue_depth`, plus HTTP request metrics middleware. Uses `metrics` facade + `metrics-exporter-prometheus`.

### ~~Admin Route Authentication~~
`/admin/v1/*` routes protected by JWT auth + bootstrap org restriction. Only users authenticated via the bootstrap organization's OIDC config can access admin endpoints.

### ~~Correlation IDs~~
`x-request-id` header middleware (generate UUID if absent, echo in response). Tracing spans on all requests and per-delivery-attempt with `message_id`, `endpoint_id`, `attempt_number`.

### ~~Idempotency Key Cleanup Job~~
Periodic `DELETE FROM messages WHERE idempotency_expires_at <= now()` in the delivery worker loop. Configurable interval via `PIGEON_WORKER_CLEANUP_INTERVAL_SECS` (default 1h).

### ~~ReplayDeadLetter Command~~
`POST /api/v1/applications/{app_id}/dead-letters/{id}/replay` — marks dead letter as replayed, creates new pending attempt. Rejects if already replayed.

## Priority: High

### `Message.delivered_at`
Denormalized timestamp updated on last successful delivery for any endpoint. Avoids expensive `SELECT MAX(attempted_at) FROM attempts WHERE message_id = $1 AND status = 'succeeded'` queries. Add to domain entity + migration.

## Priority: Medium

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
- Delivery queue full flow (dequeue + deliver + record against real Postgres)

### Signing Secret Rotation
No mechanism to rotate an endpoint's `signing_secret` without breaking in-flight deliveries. Design: dual-secret window — deliver signed with new secret, but during a configurable transition period include both old and new signatures so consumers can verify with either.

## Priority: Low

### Audit Log Postgres Implementation
`AuditStore` and `AuditBehavior` ports exist in the application layer. Needs a `PgAuditStore` adapter, `audit_log` migration, and wiring into the mediator pipeline.

### Config Crate
Currently raw `std::env::var` calls in `PigeonConfig`. Could use `config-rs` or `envy` for typed config with layered sources (env, file, defaults).

## Out of Scope (Noted)

### Inbound Webhook Signature Verification
Pigeon sends webhooks, it doesn't receive them from external services. If scope expands to receiving, inbound signatures need verification. Not currently planned.

### Frontend (pigeon-ui)
Vue 3 + TypeScript + shadcn-vue + generated OpenAPI client. Lives in `pigeon-ui/` directory. Blocked until API is stable and delivery worker is operational. OpenAPI spec already generated at `/api/openapi.json`.
