# Event Types

Event types define the categories of events an application can send. Endpoints subscribe to specific event types.

## Create

```
POST /api/v1/applications/{app_id}/event-types
```

```json
{ "name": "user.created" }
```

Names must be unique within an application.

## List

```
GET /api/v1/applications/{app_id}/event-types?offset=0&limit=20
```

## Get

```
GET /api/v1/applications/{app_id}/event-types/{id}
```

## Update

```
PUT /api/v1/applications/{app_id}/event-types/{id}
```

```json
{ "name": "user.updated", "version": 1 }
```

## Delete

```
DELETE /api/v1/applications/{app_id}/event-types/{id}
```

## Stats

```
GET /api/v1/applications/{app_id}/event-types/{id}/stats?period=24h|7d|30d
```

Returns message counts, delivery rates, subscribed endpoints, time series, and recent messages.

## System event types

Each application gets a `pigeon.test` system event type automatically. It cannot be deleted and is used by the test event endpoint.
