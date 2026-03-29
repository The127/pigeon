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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stores::MockDeadLetterReadStore;
    use pigeon_domain::dead_letter::DeadLetterState;

    #[tokio::test]
    async fn returns_dead_letter_when_found() {
        let dl = DeadLetter::reconstitute(DeadLetterState::fake());
        let id = dl.id().clone();
        let dl_clone = dl.clone();

        let mut mock = MockDeadLetterReadStore::new();
        mock.expect_find_by_id()
            .returning(move |_, _| Ok(Some(dl_clone.clone())));

        let handler = GetDeadLetterByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetDeadLetterById { id, org_id: OrganizationId::new() })
            .await
            .unwrap();

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let mut mock = MockDeadLetterReadStore::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let handler = GetDeadLetterByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetDeadLetterById {
                id: DeadLetterId::new(),
                org_id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
