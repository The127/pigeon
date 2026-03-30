# Dead Letters

Dead letters are created when a delivery attempt exhausts all retries. They preserve the last response code and body for debugging.

## List

```
GET /api/v1/applications/{app_id}/dead-letters?endpoint_id=&replayed=&offset=0&limit=50
```

Filter by `endpoint_id` and `replayed` (true/false).

## Get

```
GET /api/v1/applications/{app_id}/dead-letters/{id}
```

## Replay

```
POST /api/v1/applications/{app_id}/dead-letters/{id}/replay
```

Creates a new pending delivery attempt for the same message and endpoint. Rejects if already replayed.
