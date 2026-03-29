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
