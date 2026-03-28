use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::event_type::{EventType, EventTypeId};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::EventTypeReadStore;

#[derive(Debug)]
pub struct GetEventTypeById {
    pub id: EventTypeId,
    pub org_id: OrganizationId,
}

impl Query for GetEventTypeById {
    type Output = Option<EventType>;
}

pub struct GetEventTypeByIdHandler {
    read_store: Arc<dyn EventTypeReadStore>,
}

impl GetEventTypeByIdHandler {
    pub fn new(read_store: Arc<dyn EventTypeReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetEventTypeById> for GetEventTypeByIdHandler {
    async fn handle(
        &self,
        query: GetEventTypeById,
    ) -> Result<Option<EventType>, ApplicationError> {
        self.read_store.find_by_id(&query.id, &query.org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stores::MockEventTypeReadStore;
    use pigeon_domain::event_type::EventTypeState;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn returns_event_type_when_found() {
        let et = EventType::reconstitute(EventTypeState::fake());
        let expected_name = et.name().to_string();
        let id = et.id().clone();
        let query_id = et.id().clone();

        let mut mock = MockEventTypeReadStore::new();
        mock.expect_find_by_id()
            .withf(move |arg_id, _| *arg_id == id)
            .returning(move |_, _| Ok(Some(et.clone())));

        let handler = GetEventTypeByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetEventTypeById { id: query_id, org_id: OrganizationId::new() })
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), expected_name);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let mut mock = MockEventTypeReadStore::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let handler = GetEventTypeByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetEventTypeById {
                id: EventTypeId::new(),
                org_id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
