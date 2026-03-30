# Architecture

Hexagonal / ports-and-adapters with CQRS mediator dispatch. Compiler-enforced boundaries via Cargo workspace dependency graph.

## Crate structure

```
pigeon-macros          proc macros (Reconstitute derive)
pigeon-domain          aggregates, value objects, domain events
pigeon-application     commands, queries, ports, mediator pipeline
pigeon-infrastructure  Postgres adapters, HTTP webhook client
pigeon-api             axum handlers, JWT auth, OpenAPI
pigeon-server          composition root, CLI
pigeon-ui              Vue 3 + TypeScript frontend
```

### Dependency rules

Each crate can only depend on crates above it. Violations are compiler errors.

- **pigeon-macros** — no pigeon deps
- **pigeon-domain** — depends on pigeon-macros only
- **pigeon-application** — depends on domain only
- **pigeon-infrastructure** — depends on application + domain
- **pigeon-api** — depends on application + domain
- **pigeon-server** — depends on all (composition root)

## Data model

```
Organization (tenant)
  └── OidcConfig (issuer_url + audience → org identity)
  └── Applications
        ├── EventTypes
        ├── Endpoints → signing_secrets (max 2, for rotation)
        └── Messages → Attempts → DeadLetters
```

## Key patterns

### Mediator pipeline

Every command flows through: `TransactionBehavior → AuditBehavior → CommandHandler`

- **TransactionBehavior** creates the UoW, stores it in `RequestContext`, commits on success, rolls back on failure
- **AuditBehavior** records an audit entry after handler execution
- **CommandHandler** uses `ctx.uow()` for store operations — no manual transaction management

### Unit of Work

EF Core-style change tracker. Store methods record changes in memory, `commit()` opens a short Postgres transaction and flushes all changes. No DB transaction held during handler execution.

### Optimistic concurrency

`Version` type mapped to Postgres `xmin` system column. Updates use `WHERE xmin::text::bigint = $version` — if rows_affected = 0, return Conflict.

### Transactional outbox

Domain events are inserted into the `event_outbox` table during the same UoW commit transaction. An outbox worker polls and dispatches events to subscribers (delivery projections, auto-disable saga).

### Delivery worker

`SELECT ... FOR UPDATE SKIP LOCKED` dequeue. HMAC-SHA256 signing with all active secrets. Exponential backoff retry. Dead-lettering after exhaustion. Runs as `pigeon worker` or alongside the API via `pigeon serve`.
