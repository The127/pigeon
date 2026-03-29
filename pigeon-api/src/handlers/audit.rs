use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;

use pigeon_application::queries::list_audit_log::ListAuditLog;

use crate::dto::audit::AuditLogResponse;
use crate::dto::pagination::ListQuery;
use crate::extractors::OrgId;
use crate::state::AppState;

/// List audit log entries for the current organization
#[utoipa::path(
    get,
    path = "/api/v1/audit-log",
    params(ListQuery),
    responses(
        (status = 200, description = "Paginated list of audit log entries"),
    ),
    tag = "audit"
)]
pub async fn list_audit_log(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, crate::error::ApiError> {
    let result = state
        .list_audit_log
        .handle(ListAuditLog {
            org_id,
            offset: query.offset.unwrap_or(0),
            limit: query.limit.unwrap_or(50),
        })
        .await
        .map_err(crate::error::ApiError)?;

    let response = serde_json::json!({
        "items": result.items.into_iter().map(AuditLogResponse::from).collect::<Vec<_>>(),
        "total": result.total,
        "offset": result.offset,
        "limit": result.limit,
    });

    Ok(Json(response))
}
