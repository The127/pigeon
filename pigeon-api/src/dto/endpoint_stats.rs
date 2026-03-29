use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use pigeon_application::ports::endpoint_stats_read_store::EndpointStats;

#[derive(Debug, Serialize, ToSchema)]
pub struct EndpointStatsResponse {
    pub total_attempts: u64,
    pub total_pending: u64,
    pub total_succeeded: u64,
    pub total_failed: u64,
    pub total_dead_lettered: u64,
    pub success_rate: f64,
    pub consecutive_failures: u64,
    pub last_delivery_at: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub time_series: Vec<EndpointTimeBucketResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EndpointTimeBucketResponse {
    pub bucket: DateTime<Utc>,
    pub succeeded: u64,
    pub failed: u64,
}

impl From<EndpointStats> for EndpointStatsResponse {
    fn from(stats: EndpointStats) -> Self {
        Self {
            total_attempts: stats.total_attempts,
            total_pending: stats.total_pending,
            total_succeeded: stats.total_succeeded,
            total_failed: stats.total_failed,
            total_dead_lettered: stats.total_dead_lettered,
            success_rate: stats.success_rate,
            consecutive_failures: stats.consecutive_failures,
            last_delivery_at: stats.last_delivery_at,
            last_status: stats.last_status,
            time_series: stats
                .time_series
                .into_iter()
                .map(|b| EndpointTimeBucketResponse {
                    bucket: b.bucket,
                    succeeded: b.succeeded,
                    failed: b.failed,
                })
                .collect(),
        }
    }
}
