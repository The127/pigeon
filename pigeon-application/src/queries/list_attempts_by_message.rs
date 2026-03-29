use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::attempt::Attempt;
use pigeon_domain::message::MessageId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::AttemptReadStore;

#[derive(Debug)]
pub struct ListAttemptsByMessage {
    pub message_id: MessageId,
    pub org_id: OrganizationId,
}

impl Query for ListAttemptsByMessage {
    type Output = Vec<Attempt>;
}

pub struct ListAttemptsByMessageHandler {
    read_store: Arc<dyn AttemptReadStore>,
}

impl ListAttemptsByMessageHandler {
    pub fn new(read_store: Arc<dyn AttemptReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<ListAttemptsByMessage> for ListAttemptsByMessageHandler {
    async fn handle(
        &self,
        query: ListAttemptsByMessage,
    ) -> Result<Vec<Attempt>, ApplicationError> {
        self.read_store
            .list_by_message(&query.message_id, &query.org_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stores::MockAttemptReadStore;
    use pigeon_domain::attempt::AttemptState;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn returns_empty_list() {
        let mut mock = MockAttemptReadStore::new();
        mock.expect_list_by_message().returning(|_, _| Ok(vec![]));

        let handler = ListAttemptsByMessageHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListAttemptsByMessage {
                message_id: MessageId::new(),
                org_id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn returns_attempts() {
        let att = Attempt::reconstitute(AttemptState::fake());
        let items = vec![att];
        let items_clone = items.clone();

        let mut mock = MockAttemptReadStore::new();
        mock.expect_list_by_message()
            .returning(move |_, _| Ok(items_clone.clone()));

        let handler = ListAttemptsByMessageHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListAttemptsByMessage {
                message_id: MessageId::new(),
                org_id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
    }
}
