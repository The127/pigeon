use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::commands::replay_dead_letter::ReplayDeadLetter;
use pigeon_domain::dead_letter::DeadLetterId;

use crate::error::{ApiError, ErrorBody};
use crate::extractors::OrgId;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct ReplayDeadLetterResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub endpoint_id: Uuid,
    pub replayed_at: String,
}

/// Replay a dead-lettered message, creating a new delivery attempt
#[utoipa::path(
    post,
    path = "/api/v1/dead-letters/{id}/replay",
    params(("id" = Uuid, Path, description = "Dead letter ID")),
    responses(
        (status = 200, description = "Dead letter replayed", body = ReplayDeadLetterResponse),
        (status = 404, description = "Dead letter not found", body = ErrorBody),
        (status = 400, description = "Already replayed", body = ErrorBody),
    ),
    tag = "dead-letters"
)]
pub async fn replay(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let command = ReplayDeadLetter {
        org_id,
        dead_letter_id: DeadLetterId::from_uuid(id),
    };

    let dead_letter = state.replay_dead_letter.handle(command).await.map_err(ApiError)?;

    let response = ReplayDeadLetterResponse {
        id: *dead_letter.id().as_uuid(),
        message_id: *dead_letter.message_id().as_uuid(),
        endpoint_id: *dead_letter.endpoint_id().as_uuid(),
        replayed_at: dead_letter
            .replayed_at()
            .map(|t| t.to_rfc3339())
            .unwrap_or_default(),
    };

    Ok((StatusCode::OK, Json(response)))
}
