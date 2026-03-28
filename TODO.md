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

### ~~Manual Retry of Specific Attempt~~
`POST /api/v1/applications/{app_id}/attempts/{id}/retry` — sets status back to pending with `next_attempt_at = now()`. Only works on failed attempts.

### ~~Endpoint Test Event~~
`POST /api/v1/applications/{app_id}/endpoints/{id}/test` — sends a synthetic `pigeon.test` message to a specific endpoint. System event type auto-created per application, hidden from list/get, undeletable.

### ~~Domain Event Dispatch~~
Transactional outbox pattern: events are INSERT'd into `event_outbox` table inside the same DB transaction as domain changes. Outbox worker polls and processes events. First event: `DeadLettered` on dead letter insertion.

### ~~Integration Tests~~
Cross-tenant SQL isolation tests for applications, endpoints, event types, and OIDC configs. OidcConfig CRUD, SendMessage, and delivery queue flows were already covered.

## Priority: High (Outbox-unlocked)

### More Domain Events
Expand outbox coverage beyond `DeadLettered`:
- `MessageCreated` — on SendMessage, enables real-time message feed
- `AttemptSucceeded` / `AttemptFailed` — fine-grained delivery tracking
- `DeadLetterReplayed` — on replay, closes the loop
- `EndpointDisabled` / `EndpointEnabled` — audit trail
Emit from change tracker `collect_events()` on matching Change variants.

### Dead Letter Alert Webhooks
Subscribe to `DeadLettered` events via outbox handler. POST to a user-configurable alert URL per application. "Your endpoint X is failing" notifications without polling. First real consumer of the outbox beyond logging.

### Real-time Event Stream (SSE)
`GET /api/v1/applications/{app_id}/events/stream` — Server-Sent Events endpoint fed by the outbox worker. Frontend can subscribe for live updates on message delivery status.

## Priority: Medium

### Read Model Projections
Outbox handlers build denormalized views — e.g., "last N deliveries per endpoint" materialized table, updated by `AttemptSucceeded`/`AttemptFailed` events. Avoids expensive aggregate queries.

### Auto-disable Failing Endpoints
Saga: after N consecutive `DeadLettered` events for the same endpoint, auto-disable it. Prevents queue poisoning from permanently broken endpoints. Re-enable via API.

### Signing Secret Rotation
No mechanism to rotate an endpoint's `signing_secret` without breaking in-flight deliveries. Design: dual-secret window — deliver signed with new secret, but during a configurable transition period include both old and new signatures so consumers can verify with either.

### External Event Bus Integration
Swap outbox handler to push events to Kafka/NATS/SQS instead of (or in addition to) logging. Enables downstream systems to react to Pigeon events.

## Priority: Low

### Audit Log Postgres Implementation
`AuditStore` and `AuditBehavior` ports exist in the application layer. Needs a `PgAuditStore` adapter, `audit_log` migration, and wiring into the mediator pipeline. Could be replaced by domain events + outbox projection.

### Config Crate
Currently raw `std::env::var` calls in `PigeonConfig`. Could use `config-rs` or `envy` for typed config with layered sources (env, file, defaults).

## Out of Scope (Noted)

### Inbound Webhook Signature Verification
Pigeon sends webhooks, it doesn't receive them from external services. If scope expands to receiving, inbound signatures need verification. Not currently planned.

### Frontend (pigeon-ui)
Vue 3 + TypeScript + shadcn-vue + generated OpenAPI client. Lives in `pigeon-ui/` directory. API is stable, delivery worker is operational. OpenAPI spec at `/api/openapi.json`. Can consume real-time events via SSE once that's implemented.
