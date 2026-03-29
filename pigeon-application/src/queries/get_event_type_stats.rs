use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::event_type_stats_read_store::{EventTypeStats, EventTypeStatsReadStore};

#[derive(Debug)]
pub struct GetEventTypeStats {
    pub app_id: ApplicationId,
    pub event_type_id: EventTypeId,
    pub org_id: OrganizationId,
    pub since: DateTime<Utc>,
    pub bucket_interval_hours: u32,
}

impl Query for GetEventTypeStats {
    type Output = EventTypeStats;
}

pub struct GetEventTypeStatsHandler {
    read_store: Arc<dyn EventTypeStatsReadStore>,
}

impl GetEventTypeStatsHandler {
    pub fn new(read_store: Arc<dyn EventTypeStatsReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetEventTypeStats> for GetEventTypeStatsHandler {
    async fn handle(&self, query: GetEventTypeStats) -> Result<EventTypeStats, ApplicationError> {
        self.read_store
            .get_stats(
                &query.app_id,
                &query.event_type_id,
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
    use crate::ports::event_type_stats_read_store::MockEventTypeStatsReadStore;
    use chrono::Duration;

    #[tokio::test]
    async fn returns_stats() {
        let mut mock = MockEventTypeStatsReadStore::new();
        mock.expect_get_stats().returning(|_, _, _, _, _| {
            Ok(EventTypeStats {
                total_messages: 5,
                total_attempts: 10,
                total_pending: 1,
                total_succeeded: 8,
                total_failed: 1,
                total_dead_lettered: 0,
                success_rate: 0.8,
                subscribed_endpoints: 2,
                time_series: vec![],
                recent_messages: vec![],
            })
        });

        let handler = GetEventTypeStatsHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetEventTypeStats {
                app_id: ApplicationId::new(),
                event_type_id: EventTypeId::new(),
                org_id: OrganizationId::new(),
                since: Utc::now() - Duration::hours(24),
                bucket_interval_hours: 1,
            })
            .await
            .unwrap();

        assert_eq!(result.total_messages, 5);
        assert_eq!(result.subscribed_endpoints, 2);
    }
}
