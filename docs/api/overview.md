# API Overview

All endpoints require JWT Bearer authentication unless noted otherwise.

Base URL: `/api/v1`

## OpenAPI spec

The full OpenAPI 3.0 specification is available at:

```
GET /api/openapi.json
```

## Authentication

Include a JWT Bearer token in the `Authorization` header:

```
Authorization: Bearer <token>
```

The token must be issued by an OIDC provider registered in Pigeon. The organization is resolved from the token's `iss` and `aud` claims.

## Error format

```json
{
  "error": "Human-readable error message",
  "code": "error_code"
}
```

| Code | HTTP Status | Meaning |
|------|-------------|---------|
| `bad_request` | 400 | Validation error |
| `not_found` | 404 | Resource not found (or cross-tenant) |
| `conflict` | 409 | Optimistic concurrency conflict |
| `internal_error` | 500 | Server error |

## Pagination

List endpoints return:

```json
{
  "items": [...],
  "total": 42,
  "offset": 0,
  "limit": 20
}
```

Query params: `offset` (default 0), `limit` (default 20).

## Endpoints summary

| Resource | Methods | Path |
|----------|---------|------|
| Applications | CRUD | `/api/v1/applications` |
| Event Types | CRUD | `/api/v1/applications/{app_id}/event-types` |
| Endpoints | CRUD + rotate/revoke | `/api/v1/applications/{app_id}/endpoints` |
| Messages | Send + list | `/api/v1/applications/{app_id}/messages` |
| Attempts | List + retry | `/api/v1/applications/{app_id}/messages/{msg_id}/attempts` |
| Dead Letters | List + replay | `/api/v1/applications/{app_id}/dead-letters` |
| Stats | Per app/event-type/endpoint | `.../stats?period=24h\|7d\|30d` |
| OIDC Configs | List + create + delete | `/api/v1/oidc-configs` |
| Audit Log | List | `/api/v1/audit-log` |
