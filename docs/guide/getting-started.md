# Getting Started

## Quick start with Docker

```bash
docker pull ghcr.io/the127/pigeon-server:1
docker pull ghcr.io/the127/pigeon-ui:1
```

Run the server with a Postgres database:

```bash
docker run -d --name pigeon \
  -e DATABASE_URL=postgres://user:pass@host/pigeon \
  -e PIGEON_BOOTSTRAP_ORG_ENABLED=true \
  -e PIGEON_BOOTSTRAP_OIDC_ISSUER_URL=https://your-oidc-provider \
  -e PIGEON_BOOTSTRAP_OIDC_AUDIENCE=pigeon-api \
  -p 3000:3000 \
  ghcr.io/the127/pigeon-server:1
```

The server runs database migrations automatically on startup.

## What happens next

1. Pigeon creates a bootstrap organization with the OIDC config you provided
2. Users authenticate via your OIDC provider and get a JWT
3. Pigeon resolves the organization from the JWT's issuer + audience
4. All API operations are scoped to the authenticated organization

## Core workflow

```
1. Create an Application (logical grouping)
2. Create Event Types (e.g. "user.created", "order.placed")
3. Create Endpoints (webhook URLs subscribed to event types)
4. Send Messages (JSON payload + event type)
   → Pigeon fans out to all matching endpoints
   → Signs with HMAC-SHA256
   → Retries on failure
   → Dead-letters after exhaustion
```

## Endpoints

- **API:** `http://localhost:3000/api/v1/...`
- **OpenAPI spec:** `http://localhost:3000/api/openapi.json`
- **Health:** `http://localhost:3000/health`
- **Metrics:** `http://localhost:3000/metrics` (Prometheus format)
