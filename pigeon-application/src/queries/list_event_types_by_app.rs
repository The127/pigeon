use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventType;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::EventTypeReadStore;
use crate::queries::PaginatedResult;

#[derive(Debug)]
pub struct ListEventTypesByApp {
    pub app_id: ApplicationId,
    pub org_id: OrganizationId,
    pub offset: u64,
    pub limit: u64,
}

impl Query for ListEventTypesByApp {
    type Output = PaginatedResult<EventType>;
}

pub struct ListEventTypesByAppHandler {
    read_store: Arc<dyn EventTypeReadStore>,
}

impl ListEventTypesByAppHandler {
    pub fn new(read_store: Arc<dyn EventTypeReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<ListEventTypesByApp> for ListEventTypesByAppHandler {
    async fn handle(
        &self,
        query: ListEventTypesByApp,
    ) -> Result<PaginatedResult<EventType>, ApplicationError> {
        let items = self
            .read_store
            .list_by_app(&query.app_id, &query.org_id, query.offset, query.limit)
            .await?;
        let total = self.read_store.count_by_app(&query.app_id, &query.org_id).await?;

        Ok(PaginatedResult {
            items,
            total,
            offset: query.offset,
            limit: query.limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stores::MockEventTypeReadStore;
    use pigeon_domain::event_type::EventTypeState;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn returns_empty_list() {
        let app_id = ApplicationId::new();
        let app_id_clone = app_id.clone();

        let mut mock = MockEventTypeReadStore::new();
        mock.expect_list_by_app().returning(|_, _, _, _| Ok(vec![]));
        mock.expect_count_by_app().returning(|_, _| Ok(0));

        let handler = ListEventTypesByAppHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListEventTypesByApp {
                app_id: app_id_clone,
                org_id: OrganizationId::new(),
                offset: 0,
                limit: 10,
            })
            .await
            .unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.total, 0);
        assert_eq!(result.offset, 0);
        assert_eq!(result.limit, 10);
    }

    #[tokio::test]
    async fn returns_items_with_pagination() {
        let et1 = EventType::reconstitute(EventTypeState::fake());
        let et2 = EventType::reconstitute(EventTypeState::fake());
        let items = vec![et1, et2];
        let items_clone = items.clone();

        let mut mock = MockEventTypeReadStore::new();
        mock.expect_list_by_app()
            .withf(|_, _, offset, limit| *offset == 0 && *limit == 10)
            .returning(move |_, _, _, _| Ok(items_clone.clone()));
        mock.expect_count_by_app().returning(|_, _| Ok(5));

        let handler = ListEventTypesByAppHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListEventTypesByApp {
                app_id: ApplicationId::new(),
                org_id: OrganizationId::new(),
                offset: 0,
                limit: 10,
            })
            .await
            .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total, 5);
    }
}
