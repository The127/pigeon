# Endpoints

Endpoints are webhook URLs that receive deliveries. Each endpoint subscribes to one or more event types.

## Create

```
POST /api/v1/applications/{app_id}/endpoints
```

```json
{
  "name": "My Webhook",
  "url": "https://example.com/webhook",
  "event_type_ids": ["<uuid>"]
}
```

- `name` is optional — Pigeon generates a Docker-style name (e.g. "brave-falcon") if omitted
- `url` must use `http://` or `https://`
- Signing secret is auto-generated and returned in the response (shown once)

## List / Get / Update / Delete

Standard CRUD at `/api/v1/applications/{app_id}/endpoints[/{id}]`.

Update accepts `url` and `event_type_ids`. Signing secrets are managed separately (see below).

## Signing secret rotation

```
POST /api/v1/applications/{app_id}/endpoints/{id}/rotate
```

Returns the new secret (shown once). See [Signing Secrets](/features/signing-secrets) for the full rotation flow.

## Revoke old secret

```
DELETE /api/v1/applications/{app_id}/endpoints/{id}/secrets/{index}
```

Index 0 = newest, 1 = oldest. Cannot revoke the last remaining secret.

## Test event

```
POST /api/v1/applications/{app_id}/endpoints/{id}/test
```

Sends a synthetic `pigeon.test` event to this endpoint. Useful for verifying connectivity.

## Stats

```
GET /api/v1/applications/{app_id}/endpoints/{id}/stats?period=24h|7d|30d
```
