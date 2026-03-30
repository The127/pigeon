# Messages & Delivery

## Send a message

```
POST /api/v1/applications/{app_id}/messages
```

```json
{
  "event_type_id": "<uuid>",
  "payload": { "user_id": "123", "action": "signup" },
  "idempotency_key": "optional-unique-key"
}
```

- `payload` must be a JSON object
- `idempotency_key` is optional — if provided, duplicate sends return the existing message

Response includes `attempts_created` and `was_duplicate`.

## List messages

```
GET /api/v1/applications/{app_id}/messages?event_type_id=&status=&offset=0&limit=50
```

## Get message

```
GET /api/v1/applications/{app_id}/messages/{id}
```

Includes delivery status counts (attempts_created, succeeded, failed, dead_lettered).

## Retrigger

```
POST /api/v1/applications/{app_id}/messages/{id}/retrigger
```

Re-fans-out to currently matching endpoints, skipping those that already have attempts.

## List attempts

```
GET /api/v1/applications/{app_id}/messages/{msg_id}/attempts
```

## Retry a failed attempt

```
POST /api/v1/applications/{app_id}/attempts/{id}/retry
```

Only works on attempts with status `failed`.

## Dead letters

```
GET /api/v1/applications/{app_id}/dead-letters?endpoint_id=&replayed=&offset=0&limit=50
```

### Replay

```
POST /api/v1/applications/{app_id}/dead-letters/{id}/replay
```

Creates a new delivery attempt. Cannot replay a dead letter that was already replayed.
