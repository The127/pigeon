# Webhook Delivery

## How it works

1. You send a **message** with an event type and a JSON payload
2. Pigeon finds all **endpoints** subscribed to that event type
3. One **attempt** is created per endpoint
4. The delivery worker picks up attempts and delivers them via HTTP POST
5. On failure, attempts are retried with exponential backoff
6. After max retries, the attempt becomes a **dead letter**

## Signing

Every delivery is signed with HMAC-SHA256 using the endpoint's signing secrets:

```
X-Pigeon-Signature: sha256=<hex>
```

If the endpoint has two secrets (during rotation), both signatures are included:

```
X-Pigeon-Signature: sha256=<sig1>,sha256=<sig2>
```

Consumers should verify against any signature in the header.

## Retry behavior

| Attempt | Delay |
|---------|-------|
| 1 | Immediate |
| 2 | 30s (base) |
| 3 | 60s |
| 4 | 120s |
| 5 | 240s |

Backoff formula: `base * 2^(attempt - 2)`, capped at `max_backoff_secs`.

After `max_retries` failures, the attempt is dead-lettered.

## Dead letters

Dead-lettered messages are not lost. You can:

- **Replay** — `POST .../dead-letters/{id}/replay` creates a new attempt
- **View** — inspect the last response code and body in the UI

## Idempotency

Messages accept an optional `idempotency_key`. Sending the same key twice returns the existing message without creating duplicate attempts. Keys expire after 24 hours.

## Auto-disable

Endpoints with consecutive dead letters exceeding the threshold (`PIGEON_WORKER_AUTO_DISABLE_THRESHOLD`, default 5) are automatically disabled. Set to 0 to turn off.

## Test events

`POST .../endpoints/{id}/test` sends a synthetic `pigeon.test` event to a single endpoint. Useful for verifying connectivity and signature verification.
