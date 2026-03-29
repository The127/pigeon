use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::message_status::MessageWithStatus;
use crate::ports::stores::MessageReadStore;
use crate::queries::PaginatedResult;

#[derive(Debug)]
pub struct ListMessagesByApp {
    pub app_id: ApplicationId,
    pub org_id: OrganizationId,
    pub offset: u64,
    pub limit: u64,
}

impl Query for ListMessagesByApp {
    type Output = PaginatedResult<MessageWithStatus>;
}

pub struct ListMessagesByAppHandler {
    read_store: Arc<dyn MessageReadStore>,
}

impl ListMessagesByAppHandler {
    pub fn new(read_store: Arc<dyn MessageReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<ListMessagesByApp> for ListMessagesByAppHandler {
    async fn handle(
        &self,
        query: ListMessagesByApp,
    ) -> Result<PaginatedResult<MessageWithStatus>, ApplicationError> {
        let items = self
            .read_store
            .list_by_app(&query.app_id, &query.org_id, query.offset, query.limit)
            .await?;
        let total = self
            .read_store
            .count_by_app(&query.app_id, &query.org_id)
            .await?;

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
    use crate::ports::stores::MockMessageReadStore;
    use pigeon_domain::message::{Message, MessageState};

    fn fake_msg_with_status() -> MessageWithStatus {
        MessageWithStatus {
            message: Message::reconstitute(MessageState::fake()),
            attempts_created: 1,
            succeeded: 0,
            failed: 0,
            dead_lettered: 0,
        }
    }

    #[tokio::test]
    async fn returns_empty_list() {
        let mut mock = MockMessageReadStore::new();
        mock.expect_list_by_app().returning(|_, _, _, _| Ok(vec![]));
        mock.expect_count_by_app().returning(|_, _| Ok(0));

        let handler = ListMessagesByAppHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListMessagesByApp {
                app_id: ApplicationId::new(),
                org_id: OrganizationId::new(),
                offset: 0,
                limit: 10,
            })
            .await
            .unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.total, 0);
    }

    #[tokio::test]
    async fn returns_items_with_pagination() {
        let mws = fake_msg_with_status();
        let items = vec![mws];
        let items_clone = items.clone();

        let mut mock = MockMessageReadStore::new();
        mock.expect_list_by_app()
            .returning(move |_, _, _, _| Ok(items_clone.clone()));
        mock.expect_count_by_app().returning(|_, _| Ok(5));

        let handler = ListMessagesByAppHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListMessagesByApp {
                app_id: ApplicationId::new(),
                org_id: OrganizationId::new(),
                offset: 0,
                limit: 10,
            })
            .await
            .unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.total, 5);
    }
}
