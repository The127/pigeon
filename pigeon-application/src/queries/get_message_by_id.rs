use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::message::MessageId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::message_status::MessageWithStatus;
use crate::ports::stores::MessageReadStore;

#[derive(Debug)]
pub struct GetMessageById {
    pub id: MessageId,
    pub org_id: OrganizationId,
}

impl Query for GetMessageById {
    type Output = Option<MessageWithStatus>;
}

pub struct GetMessageByIdHandler {
    read_store: Arc<dyn MessageReadStore>,
}

impl GetMessageByIdHandler {
    pub fn new(read_store: Arc<dyn MessageReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetMessageById> for GetMessageByIdHandler {
    async fn handle(
        &self,
        query: GetMessageById,
    ) -> Result<Option<MessageWithStatus>, ApplicationError> {
        self.read_store.find_by_id(&query.id, &query.org_id).await
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
            attempts_created: 2,
            succeeded: 1,
            failed: 1,
            dead_lettered: 0,
        }
    }

    #[tokio::test]
    async fn returns_message_when_found() {
        let mws = fake_msg_with_status();
        let id = mws.message.id().clone();
        let mws_clone = mws.clone();

        let mut mock = MockMessageReadStore::new();
        mock.expect_find_by_id()
            .returning(move |_, _| Ok(Some(mws_clone.clone())));

        let handler = GetMessageByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetMessageById { id, org_id: OrganizationId::new() })
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().attempts_created, 2);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let mut mock = MockMessageReadStore::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let handler = GetMessageByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetMessageById {
                id: MessageId::new(),
                org_id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
