# Applications

An application is the top-level grouping for event types, endpoints, and messages.

## Create

```
POST /api/v1/applications
```

```json
{ "name": "My App", "uid": "my-app" }
```

## List

```
GET /api/v1/applications?search=&offset=0&limit=20
```

## Get

```
GET /api/v1/applications/{id}
```

## Update

```
PUT /api/v1/applications/{id}
```

```json
{ "name": "New Name", "version": 1 }
```

Requires the current `version` for optimistic concurrency.

## Delete

```
DELETE /api/v1/applications/{id}
```

Cascades to all child entities (event types, endpoints, messages, attempts, dead letters).

## Stats

```
GET /api/v1/applications/{id}/stats?period=24h|7d|30d
```

Returns aggregate delivery counts and a time-bucketed chart.
