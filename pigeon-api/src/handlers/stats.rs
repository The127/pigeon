use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{Duration, Utc};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use pigeon_application::queries::get_app_stats::GetAppStats;
use pigeon_domain::application::ApplicationId;

use crate::dto::stats::AppStatsResponse;
use crate::error::{ApiError, ErrorBody};
use crate::extractors::OrgId;
use crate::state::AppState;

use super::verify_app_ownership;

#[derive(Debug, Deserialize, IntoParams)]
pub struct StatsQuery {
    /// Time period: "24h", "7d", or "30d"
    pub period: Option<String>,
}

/// Get delivery statistics for an application
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/stats",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        StatsQuery,
    ),
    responses(
        (status = 200, description = "Application statistics", body = AppStatsResponse),
        (status = 404, description = "Application not found", body = ErrorBody),
    ),
    tag = "applications"
)]
pub async fn get_stats(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(app_id): Path<Uuid>,
    Query(query): Query<StatsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let period = query.period.as_deref().unwrap_or("24h");
    let (since, bucket_interval_hours) = match period {
        "7d" => (Utc::now() - Duration::days(7), 6),
        "30d" => (Utc::now() - Duration::days(30), 24),
        _ => (Utc::now() - Duration::hours(24), 1), // default 24h
    };

    let stats = state
        .get_app_stats
        .handle(GetAppStats {
            app_id,
            org_id,
            since,
            bucket_interval_hours,
        })
        .await
        .map_err(ApiError)?;

    Ok(Json(AppStatsResponse::from(stats)))
}
