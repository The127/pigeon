# Local Setup

## Prerequisites

- Rust (stable)
- Node.js 22+
- Docker (for Postgres)
- [just](https://github.com/casey/just) (task runner)

## Backend

```bash
# Start Postgres
just dev-up

# Run migrations + start server
just dev-run

# Or watch mode
just dev-watch
```

## Frontend

```bash
cd pigeon-ui
npm install
npm run dev
```

The Vite dev server proxies `/api`, `/admin`, `/health`, and `/metrics` to `localhost:3000`.

## Generate OpenAPI client

Requires the backend running on port 3000:

```bash
cd pigeon-ui
npm run generate-api
```

## Testing

```bash
just test       # all tests
just clippy     # lint (warnings = errors)
just fmt-check  # format check
just ci         # fmt + clippy + test
```

## Project layout

```
pigeon-macros/          proc-macro crate
pigeon-domain/          domain model + test support
pigeon-application/     commands, queries, ports, mediator
  tests/                BDD features + step definitions
pigeon-infrastructure/  Postgres + HTTP adapters
  migrations/           SQL migrations (sqlx)
pigeon-api/             axum HTTP layer
pigeon-server/          composition root + CLI
pigeon-ui/              Vue 3 frontend
docs/                   this documentation (VitePress)
```
