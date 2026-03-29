use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stats_read_store::{AppStats, StatsReadStore};

#[derive(Debug)]
pub struct GetAppStats {
    pub app_id: ApplicationId,
    pub org_id: OrganizationId,
    pub since: DateTime<Utc>,
    pub bucket_interval_hours: u32,
}

impl Query for GetAppStats {
    type Output = AppStats;
}

pub struct GetAppStatsHandler {
    read_store: Arc<dyn StatsReadStore>,
}

impl GetAppStatsHandler {
    pub fn new(read_store: Arc<dyn StatsReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetAppStats> for GetAppStatsHandler {
    async fn handle(&self, query: GetAppStats) -> Result<AppStats, ApplicationError> {
        self.read_store
            .get_app_stats(
                &query.app_id,
                &query.org_id,
                query.since,
                query.bucket_interval_hours,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stats_read_store::{MockStatsReadStore, TimeBucket};
    use chrono::Duration;

    #[tokio::test]
    async fn returns_stats() {
        let mut mock = MockStatsReadStore::new();
        mock.expect_get_app_stats().returning(|_, _, _, _| {
            Ok(AppStats {
                total_messages: 10,
                total_attempts: 20,
                total_pending: 0,
                total_succeeded: 18,
                total_failed: 2,
                total_dead_lettered: 1,
                success_rate: 0.9,
                time_series: vec![TimeBucket {
                    bucket: Utc::now(),
                    succeeded: 18,
                    failed: 2,
                }],
            })
        });

        let handler = GetAppStatsHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetAppStats {
                app_id: ApplicationId::new(),
                org_id: OrganizationId::new(),
                since: Utc::now() - Duration::hours(24),
                bucket_interval_hours: 1,
            })
            .await
            .unwrap();

        assert_eq!(result.total_messages, 10);
        assert_eq!(result.total_succeeded, 18);
        assert_eq!(result.time_series.len(), 1);
    }
}
