use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::commands::retry_attempt::RetryAttempt;
use pigeon_domain::attempt::AttemptId;

use crate::error::{ApiError, ErrorBody};
use crate::extractors::OrgId;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct RetryAttemptResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub endpoint_id: Uuid,
    pub status: String,
}

/// Retry a failed delivery attempt immediately
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/attempts/{id}/retry",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Attempt ID"),
    ),
    responses(
        (status = 200, description = "Attempt retried", body = RetryAttemptResponse),
        (status = 404, description = "Attempt not found", body = ErrorBody),
        (status = 400, description = "Attempt is not in failed state", body = ErrorBody),
    ),
    tag = "attempts"
)]
pub async fn retry(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((_app_id, attempt_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let command = RetryAttempt {
        org_id,
        attempt_id: AttemptId::from_uuid(attempt_id),
    };

    let attempt = state.retry_attempt.handle(command).await.map_err(ApiError)?;

    let status_str = match attempt.status() {
        pigeon_domain::attempt::AttemptStatus::Pending => "pending",
        pigeon_domain::attempt::AttemptStatus::InFlight => "in_flight",
        pigeon_domain::attempt::AttemptStatus::Succeeded => "succeeded",
        pigeon_domain::attempt::AttemptStatus::Failed => "failed",
    };

    let response = RetryAttemptResponse {
        id: *attempt.id().as_uuid(),
        message_id: *attempt.message_id().as_uuid(),
        endpoint_id: *attempt.endpoint_id().as_uuid(),
        status: status_str.to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}
