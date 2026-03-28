use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::commands::send_test_event::SendTestEvent;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::EndpointId;

use crate::error::{ApiError, ErrorBody};
use crate::extractors::OrgId;
use crate::state::AppState;

use super::verify_app_ownership;

#[derive(Serialize, ToSchema)]
pub struct TestEventResponse {
    pub message_id: Uuid,
}

/// Send a test event to a specific endpoint
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/endpoints/{id}/test",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Endpoint ID"),
    ),
    responses(
        (status = 201, description = "Test event sent", body = TestEventResponse),
        (status = 404, description = "Application or endpoint not found", body = ErrorBody),
    ),
    tag = "endpoints"
)]
pub async fn send_test_event(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, endpoint_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let command = SendTestEvent {
        org_id,
        app_id,
        endpoint_id: EndpointId::from_uuid(endpoint_id),
    };

    let result = state.send_test_event.handle(command).await.map_err(ApiError)?;

    let response = TestEventResponse {
        message_id: *result.message.id().as_uuid(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}
