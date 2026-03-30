# Configuration

All configuration is via environment variables. Application variables use the `PIGEON_` prefix.

## Environment variables

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

```bash
pigeon serve    # API + worker (default in Docker)
pigeon api      # API only
pigeon worker   # Worker only
pigeon migrate  # Run migrations and exit
```

Split `api` and `worker` for independent scaling. The worker can run multiple instances — `FOR UPDATE SKIP LOCKED` prevents duplicate delivery.
