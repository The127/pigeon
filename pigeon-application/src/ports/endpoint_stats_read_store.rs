use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct EndpointStats {
    pub total_attempts: u64,
    pub total_pending: u64,
    pub total_succeeded: u64,
    pub total_failed: u64,
    pub total_dead_lettered: u64,
    pub success_rate: f64,
    pub consecutive_failures: u64,
    pub last_delivery_at: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub time_series: Vec<EndpointTimeBucket>,
}

#[derive(Debug, Clone)]
pub struct EndpointTimeBucket {
    pub bucket: DateTime<Utc>,
    pub succeeded: u64,
    pub failed: u64,
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait EndpointStatsReadStore: Send + Sync {
    async fn get_stats(
        &self,
        endpoint_id: &EndpointId,
        org_id: &OrganizationId,
        since: DateTime<Utc>,
        bucket_interval_hours: u32,
    ) -> Result<EndpointStats, ApplicationError>;
}
