# Pigeon — Backlog

## Done

### ~~Delivery Worker~~
SKIP LOCKED dequeue, HTTP POST, exponential backoff retry, dead lettering.

### ~~HMAC Payload Signing~~
`X-Pigeon-Signature: sha256={hex}` via HMAC-SHA256. Optional — endpoints without a signing secret skip the header.

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
Transactional outbox pattern with always-explicit event emission. Handlers emit events via `uow.emit_event()`, delivery queue emits directly to outbox. Change tracker only collects explicit events. Events: `MessageCreated`, `MessageRetriggered`, `AttemptSucceeded`, `AttemptFailed`, `DeadLettered`, `DeadLetterReplayed`, `EndpointUpdated`.

### ~~Integration Tests~~
Cross-tenant SQL isolation tests for applications, endpoints, event types, and OIDC configs. OidcConfig CRUD, SendMessage, and delivery queue flows were already covered.

### ~~Read Model Projections~~
`endpoint_delivery_summary` (success/failure counts, consecutive failures, last delivery) and `message_delivery_status` (attempts created, succeeded, failed, dead lettered). Maintained by `DeliveryProjectionSubscriber` via outbox events. Updated on retrigger via `MessageRetriggered` event.

### ~~Auto-disable Failing Endpoints~~
`AutoDisableEndpointSaga` — outbox event subscriber that listens for `DeadLettered`, queries consecutive failure count, disables endpoint via `DisableEndpoint` command when threshold reached. Configurable via `PIGEON_WORKER_AUTO_DISABLE_THRESHOLD` (default 5, 0 to disable).

### ~~Organization requires OIDC config~~
`CreateOrganization` requires OIDC issuer + audience. `DeleteOidcConfig` rejects deleting the last config. Bootstrap requires OIDC env vars.

### ~~JWT Algorithm Support~~
Auth middleware reads algorithm from JWT header instead of hardcoding RS256. Supports EdDSA (Ed25519) and all `jsonwebtoken` algorithms.

### ~~Message Read API~~
- `GET /api/v1/applications/{app_id}/messages` — list messages with delivery status (paginated)
- `GET /api/v1/applications/{app_id}/messages/{id}` — get message by ID with delivery status

### ~~Attempt Read API~~
- `GET /api/v1/applications/{app_id}/messages/{msg_id}/attempts` — list attempts for a message

### ~~Dead Letter Read API~~
- `GET /api/v1/applications/{app_id}/dead-letters` — list dead letters (paginated)
- `GET /api/v1/applications/{app_id}/dead-letters/{id}` — get dead letter by ID

### ~~Application Stats API~~
`GET /api/v1/applications/{app_id}/stats?period=24h|7d|30d` — aggregate counts + time-bucketed delivery chart.

### ~~Event Type Stats API~~
`GET /api/v1/applications/{app_id}/event-types/{id}/stats?period=24h|7d|30d` — per-event-type metrics: message count, delivery rate, subscribed endpoints, time series, recent messages.

### ~~RetriggerMessage Command~~
`POST /api/v1/applications/{app_id}/messages/{id}/retrigger` — re-fans-out to currently matching endpoints, skipping those that already have attempts. Emits `MessageRetriggered` event to update projections.

### ~~Frontend (pigeon-ui)~~
Vue 3 + TypeScript + Tailwind v4 + shadcn-vue + TanStack Query + oidc-client-ts. OIDC auth with route guard, generated OpenAPI client. Features:
- App shell with collapsible sidebar, Pigeon logo
- Login page with split layout, health check, feature highlights
- Applications list with create/delete, auto-slug UID
- Application detail with tabbed view: Dashboard (stats + chart), Event Types, Endpoints, Messages, Dead Letters, Send Message
- Event type detail page with stats dashboard, edit/delete, subscribed endpoints, recent messages
- Messages with expandable delivery attempts, retrigger button
- Dead letters with replay button
- Endpoints with auto-generated names (Docker-style), optional signing secret
- Reusable components: PageHeader, EmptyState, FormField, LoadingState, ErrorState, StatCard, DeliveryChart
- Test webhook endpoint (`just dev-endpoint`)
- `ON DELETE CASCADE` for all FK constraints

## Priority: High (Outbox-unlocked)

### Dead Letter Alert Webhooks
Subscribe to `DeadLettered` events via outbox handler. POST to a user-configurable alert URL per application. "Your endpoint X is failing" notifications without polling. First real consumer of the outbox beyond logging.

### Real-time Event Stream (SSE)
`GET /api/v1/applications/{app_id}/events/stream` — Server-Sent Events endpoint fed by the outbox worker. Frontend can subscribe for live updates on message delivery status.

## Priority: Medium

### Frontend: Polish
- Toast notifications for mutation success/error
- Dark mode toggle
- Mobile responsive sidebar (sheet overlay on small screens)
- Edit endpoint (URL, name, signing secret, event type subscriptions)

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
