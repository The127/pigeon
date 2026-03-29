use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::commands::replay_dead_letter::ReplayDeadLetter;
use pigeon_application::queries::get_dead_letter_by_id::GetDeadLetterById;
use pigeon_application::queries::list_dead_letters_by_app::ListDeadLettersByApp;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::dead_letter::DeadLetterId;

use crate::dto::dead_letter::DeadLetterResponse;
use crate::dto::pagination::ListQuery;
use crate::error::{ApiError, ErrorBody};
use crate::extractors::{AuthInfo, OrgId};
use crate::state::AppState;
use pigeon_application::mediator::dispatcher::dispatch;

use super::verify_app_ownership;

#[derive(Serialize, ToSchema)]
pub struct ReplayDeadLetterResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub endpoint_id: Uuid,
    pub replayed_at: String,
}

/// List dead letters for an application
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/dead-letters",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ListQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of dead letters"),
    ),
    tag = "dead-letters"
)]
pub async fn list_dead_letters(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(app_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let result = state
        .list_dead_letters
        .handle(ListDeadLettersByApp {
            app_id,
            org_id,
            offset: query.offset.unwrap_or(0),
            limit: query.limit.unwrap_or(20),
        })
        .await
        .map_err(ApiError)?;

    let response = serde_json::json!({
        "items": result.items.into_iter().map(DeadLetterResponse::from).collect::<Vec<_>>(),
        "total": result.total,
        "offset": result.offset,
        "limit": result.limit,
    });

    Ok(Json(response))
}

/// Get a dead letter by ID
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/dead-letters/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Dead Letter ID"),
    ),
    responses(
        (status = 200, description = "Dead letter found", body = DeadLetterResponse),
        (status = 404, description = "Dead letter not found", body = ErrorBody),
    ),
    tag = "dead-letters"
)]
pub async fn get_dead_letter(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let dl = state
        .get_dead_letter
        .handle(GetDeadLetterById {
            id: DeadLetterId::from_uuid(id),
            org_id,
        })
        .await
        .map_err(ApiError)?
        .ok_or(ApiError(pigeon_application::error::ApplicationError::NotFound))?;

    Ok(Json(DeadLetterResponse::from(dl)))
}

/// Replay a dead-lettered message, creating a new delivery attempt
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/dead-letters/{id}/replay",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Dead letter ID"),
    ),
    responses(
        (status = 200, description = "Dead letter replayed", body = ReplayDeadLetterResponse),
        (status = 404, description = "Dead letter not found", body = ErrorBody),
        (status = 400, description = "Already replayed", body = ErrorBody),
    ),
    tag = "dead-letters"
)]
pub async fn replay(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path((_app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let command = ReplayDeadLetter {
        org_id: auth.org_id.clone(),
        dead_letter_id: DeadLetterId::from_uuid(id),
    };

    let dead_letter = dispatch(&*state.replay_dead_letter, command, &auth.user_id, &auth.org_id, &*state.audit_store).await.map_err(ApiError)?;

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
