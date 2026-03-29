use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::ports::event_type_stats_read_store::EventTypeStats;

#[derive(Debug, Serialize, ToSchema)]
pub struct EventTypeStatsResponse {
    pub total_messages: u64,
    pub total_attempts: u64,
    pub total_pending: u64,
    pub total_succeeded: u64,
    pub total_failed: u64,
    pub total_dead_lettered: u64,
    pub success_rate: f64,
    pub subscribed_endpoints: u64,
    pub time_series: Vec<EventTypeTimeBucketResponse>,
    pub recent_messages: Vec<RecentMessageResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventTypeTimeBucketResponse {
    pub bucket: DateTime<Utc>,
    pub succeeded: u64,
    pub failed: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecentMessageResponse {
    pub id: Uuid,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
    pub attempts_created: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub dead_lettered: u32,
}

impl From<EventTypeStats> for EventTypeStatsResponse {
    fn from(stats: EventTypeStats) -> Self {
        Self {
            total_messages: stats.total_messages,
            total_attempts: stats.total_attempts,
            total_pending: stats.total_pending,
            total_succeeded: stats.total_succeeded,
            total_failed: stats.total_failed,
            total_dead_lettered: stats.total_dead_lettered,
            success_rate: stats.success_rate,
            subscribed_endpoints: stats.subscribed_endpoints,
            time_series: stats
                .time_series
                .into_iter()
                .map(|b| EventTypeTimeBucketResponse {
                    bucket: b.bucket,
                    succeeded: b.succeeded,
                    failed: b.failed,
                })
                .collect(),
            recent_messages: stats
                .recent_messages
                .into_iter()
                .map(|m| RecentMessageResponse {
                    id: m.id,
                    idempotency_key: m.idempotency_key,
                    created_at: m.created_at,
                    attempts_created: m.attempts_created,
                    succeeded: m.succeeded,
                    failed: m.failed,
                    dead_lettered: m.dead_lettered,
                })
                .collect(),
        }
    }
}
