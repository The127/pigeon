use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::endpoint_stats_read_store::{EndpointStats, EndpointStatsReadStore};

#[derive(Debug)]
pub struct GetEndpointStats {
    pub endpoint_id: EndpointId,
    pub org_id: OrganizationId,
    pub since: DateTime<Utc>,
    pub bucket_interval_hours: u32,
}

impl Query for GetEndpointStats {
    type Output = EndpointStats;
}

pub struct GetEndpointStatsHandler {
    read_store: Arc<dyn EndpointStatsReadStore>,
}

impl GetEndpointStatsHandler {
    pub fn new(read_store: Arc<dyn EndpointStatsReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetEndpointStats> for GetEndpointStatsHandler {
    async fn handle(&self, query: GetEndpointStats) -> Result<EndpointStats, ApplicationError> {
        self.read_store
            .get_stats(
                &query.endpoint_id,
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
    use crate::ports::endpoint_stats_read_store::MockEndpointStatsReadStore;
    use chrono::Duration;

    #[tokio::test]
    async fn returns_stats() {
        let mut mock = MockEndpointStatsReadStore::new();
        mock.expect_get_stats().returning(|_, _, _, _| {
            Ok(EndpointStats {
                total_attempts: 10,
                total_pending: 0,
                total_succeeded: 9,
                total_failed: 1,
                total_dead_lettered: 0,
                success_rate: 0.9,
                consecutive_failures: 0,
                last_delivery_at: Some(Utc::now()),
                last_status: Some("succeeded".into()),
                time_series: vec![],
            })
        });

        let handler = GetEndpointStatsHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetEndpointStats {
                endpoint_id: EndpointId::new(),
                org_id: OrganizationId::new(),
                since: Utc::now() - Duration::hours(24),
                bucket_interval_hours: 1,
            })
            .await
            .unwrap();

        assert_eq!(result.total_attempts, 10);
        assert_eq!(result.consecutive_failures, 0);
    }
}
