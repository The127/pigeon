use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{Duration, Utc};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use pigeon_application::queries::get_event_type_stats::GetEventTypeStats;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventTypeId;

use crate::dto::event_type_stats::EventTypeStatsResponse;
use crate::error::{ApiError, ErrorBody};
use crate::extractors::OrgId;
use crate::state::AppState;

use super::verify_app_ownership;

#[derive(Debug, Deserialize, IntoParams)]
pub struct EventTypeStatsQuery {
    pub period: Option<String>,
}

/// Get delivery statistics for an event type
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/event-types/{id}/stats",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Event Type ID"),
        EventTypeStatsQuery,
    ),
    responses(
        (status = 200, description = "Event type statistics", body = EventTypeStatsResponse),
        (status = 404, description = "Application not found", body = ErrorBody),
    ),
    tag = "event-types"
)]
pub async fn get_event_type_stats(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
    Query(query): Query<EventTypeStatsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let period = query.period.as_deref().unwrap_or("24h");
    let (since, bucket_interval_hours) = match period {
        "7d" => (Utc::now() - Duration::days(7), 6),
        "30d" => (Utc::now() - Duration::days(30), 24),
        _ => (Utc::now() - Duration::hours(24), 1),
    };

    let stats = state
        .get_event_type_stats
        .handle(GetEventTypeStats {
            app_id,
            event_type_id: EventTypeId::from_uuid(id),
            org_id,
            since,
            bucket_interval_hours,
        })
        .await
        .map_err(ApiError)?;

    Ok(Json(EventTypeStatsResponse::from(stats)))
}
