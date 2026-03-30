# Deployment

## Docker images

```bash
docker pull ghcr.io/the127/pigeon-server:1
docker pull ghcr.io/the127/pigeon-ui:1
```

### Server

The server image runs `pigeon serve` by default (API + worker). Override the command for split deployments:

```bash
# API only
docker run ghcr.io/the127/pigeon-server:1 api

# Worker only
docker run ghcr.io/the127/pigeon-server:1 worker

# Run migrations only
docker run ghcr.io/the127/pigeon-server:1 migrate
```

### UI

The UI image serves the Vue SPA via nginx and proxies API requests to `pigeon-server:3000`. In Docker Compose or Kubernetes, ensure the UI container can reach the server at that hostname.

## Docker Compose example

```yaml
services:
  postgres:
    image: postgres:17-alpine
    environment:
      POSTGRES_USER: pigeon
      POSTGRES_PASSWORD: pigeon
      POSTGRES_DB: pigeon
    volumes:
      - pgdata:/var/lib/postgresql/data

  pigeon-server:
    image: ghcr.io/the127/pigeon-server:1
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgres://pigeon:pigeon@postgres/pigeon
      PIGEON_BOOTSTRAP_ORG_ENABLED: "true"
      PIGEON_BOOTSTRAP_OIDC_ISSUER_URL: https://your-oidc-provider
      PIGEON_BOOTSTRAP_OIDC_AUDIENCE: pigeon-api
    ports:
      - "3000:3000"

  pigeon-ui:
    image: ghcr.io/the127/pigeon-ui:1
    depends_on:
      - pigeon-server
    ports:
      - "8080:80"

volumes:
  pgdata:
```

## Requirements

- **Postgres 14+** — tested with PostgreSQL 17, uses standard features (`xmin`, `TEXT[]`, `ON DELETE CASCADE`)
- **OIDC provider** — any OpenID Connect provider that issues JWTs (Keycloak, Auth0, Keyline, etc.)
