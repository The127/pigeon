use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::dead_letter::DeadLetter;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::DeadLetterReadStore;
use crate::queries::PaginatedResult;

#[derive(Debug)]
pub struct ListDeadLettersByApp {
    pub app_id: ApplicationId,
    pub org_id: OrganizationId,
    pub offset: u64,
    pub limit: u64,
}

impl Query for ListDeadLettersByApp {
    type Output = PaginatedResult<DeadLetter>;
}

pub struct ListDeadLettersByAppHandler {
    read_store: Arc<dyn DeadLetterReadStore>,
}

impl ListDeadLettersByAppHandler {
    pub fn new(read_store: Arc<dyn DeadLetterReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<ListDeadLettersByApp> for ListDeadLettersByAppHandler {
    async fn handle(
        &self,
        query: ListDeadLettersByApp,
    ) -> Result<PaginatedResult<DeadLetter>, ApplicationError> {
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
    use crate::ports::stores::MockDeadLetterReadStore;

    #[tokio::test]
    async fn returns_empty_list() {
        let mut mock = MockDeadLetterReadStore::new();
        mock.expect_list_by_app().returning(|_, _, _, _| Ok(vec![]));
        mock.expect_count_by_app().returning(|_, _| Ok(0));

        let handler = ListDeadLettersByAppHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListDeadLettersByApp {
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
}
