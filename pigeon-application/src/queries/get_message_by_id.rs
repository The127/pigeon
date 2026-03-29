use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::message::{Message, MessageId};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::MessageReadStore;

#[derive(Debug)]
pub struct GetMessageById {
    pub id: MessageId,
    pub org_id: OrganizationId,
}

impl Query for GetMessageById {
    type Output = Option<Message>;
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
    ) -> Result<Option<Message>, ApplicationError> {
        self.read_store.find_by_id(&query.id, &query.org_id).await
    }
}
