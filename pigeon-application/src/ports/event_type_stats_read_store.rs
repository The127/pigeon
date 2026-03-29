use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct EventTypeStats {
    pub total_messages: u64,
    pub total_attempts: u64,
    pub total_pending: u64,
    pub total_succeeded: u64,
    pub total_failed: u64,
    pub total_dead_lettered: u64,
    pub success_rate: f64,
    pub subscribed_endpoints: u64,
    pub time_series: Vec<EventTypeTimeBucket>,
    pub recent_messages: Vec<RecentMessage>,
}

#[derive(Debug, Clone)]
pub struct EventTypeTimeBucket {
    pub bucket: DateTime<Utc>,
    pub succeeded: u64,
    pub failed: u64,
}

#[derive(Debug, Clone)]
pub struct RecentMessage {
    pub id: uuid::Uuid,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
    pub attempts_created: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub dead_lettered: u32,
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait EventTypeStatsReadStore: Send + Sync {
    async fn get_stats(
        &self,
        app_id: &ApplicationId,
        event_type_id: &EventTypeId,
        org_id: &OrganizationId,
        since: DateTime<Utc>,
        bucket_interval_hours: u32,
    ) -> Result<EventTypeStats, ApplicationError>;
}
