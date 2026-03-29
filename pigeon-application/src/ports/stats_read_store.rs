use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct AppStats {
    pub total_messages: u64,
    pub total_attempts: u64,
    pub total_succeeded: u64,
    pub total_failed: u64,
    pub total_dead_lettered: u64,
    pub success_rate: f64,
    pub time_series: Vec<TimeBucket>,
}

#[derive(Debug, Clone)]
pub struct TimeBucket {
    pub bucket: DateTime<Utc>,
    pub succeeded: u64,
    pub failed: u64,
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait StatsReadStore: Send + Sync {
    async fn get_app_stats(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        since: DateTime<Utc>,
        bucket_interval_hours: u32,
    ) -> Result<AppStats, ApplicationError>;
}
