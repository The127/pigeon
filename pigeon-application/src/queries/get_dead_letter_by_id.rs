use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::dead_letter::{DeadLetter, DeadLetterId};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::DeadLetterReadStore;

#[derive(Debug)]
pub struct GetDeadLetterById {
    pub id: DeadLetterId,
    pub org_id: OrganizationId,
}

impl Query for GetDeadLetterById {
    type Output = Option<DeadLetter>;
}

pub struct GetDeadLetterByIdHandler {
    read_store: Arc<dyn DeadLetterReadStore>,
}

impl GetDeadLetterByIdHandler {
    pub fn new(read_store: Arc<dyn DeadLetterReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetDeadLetterById> for GetDeadLetterByIdHandler {
    async fn handle(
        &self,
        query: GetDeadLetterById,
    ) -> Result<Option<DeadLetter>, ApplicationError> {
        self.read_store.find_by_id(&query.id, &query.org_id).await
    }
}
