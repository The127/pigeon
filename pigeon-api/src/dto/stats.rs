use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use pigeon_application::ports::stats_read_store::AppStats;

#[derive(Debug, Serialize, ToSchema)]
pub struct AppStatsResponse {
    pub total_messages: u64,
    pub total_attempts: u64,
    pub total_pending: u64,
    pub total_succeeded: u64,
    pub total_failed: u64,
    pub total_dead_lettered: u64,
    pub success_rate: f64,
    pub time_series: Vec<TimeBucketResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TimeBucketResponse {
    pub bucket: DateTime<Utc>,
    pub succeeded: u64,
    pub failed: u64,
}

impl From<AppStats> for AppStatsResponse {
    fn from(stats: AppStats) -> Self {
        Self {
            total_messages: stats.total_messages,
            total_attempts: stats.total_attempts,
            total_pending: stats.total_pending,
            total_succeeded: stats.total_succeeded,
            total_failed: stats.total_failed,
            total_dead_lettered: stats.total_dead_lettered,
            success_rate: stats.success_rate,
            time_series: stats
                .time_series
                .into_iter()
                .map(|b| TimeBucketResponse {
                    bucket: b.bucket,
                    succeeded: b.succeeded,
                    failed: b.failed,
                })
                .collect(),
        }
    }
}
