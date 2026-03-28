use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::organization::Organization;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::OrganizationReadStore;
use crate::queries::PaginatedResult;

#[derive(Debug)]
pub struct ListOrganizations {
    pub offset: u64,
    pub limit: u64,
}

impl Query for ListOrganizations {
    type Output = PaginatedResult<Organization>;
}

pub struct ListOrganizationsHandler {
    read_store: Arc<dyn OrganizationReadStore>,
}

impl ListOrganizationsHandler {
    pub fn new(read_store: Arc<dyn OrganizationReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<ListOrganizations> for ListOrganizationsHandler {
    async fn handle(
        &self,
        query: ListOrganizations,
    ) -> Result<PaginatedResult<Organization>, ApplicationError> {
        let items = self.read_store.list(query.offset, query.limit).await?;
        let total = self.read_store.count().await?;

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
    use crate::ports::stores::MockOrganizationReadStore;
    use pigeon_domain::organization::OrganizationState;

    #[tokio::test]
    async fn returns_empty_list() {
        let mut mock = MockOrganizationReadStore::new();
        mock.expect_list().returning(|_, _| Ok(vec![]));
        mock.expect_count().returning(|| Ok(0));

        let handler = ListOrganizationsHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListOrganizations {
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
        let org1 = Organization::reconstitute(OrganizationState::fake());
        let org2 = Organization::reconstitute(OrganizationState::fake());
        let items = vec![org1, org2];
        let items_clone = items.clone();

        let mut mock = MockOrganizationReadStore::new();
        mock.expect_list()
            .withf(|offset, limit| *offset == 0 && *limit == 10)
            .returning(move |_, _| Ok(items_clone.clone()));
        mock.expect_count().returning(|| Ok(5));

        let handler = ListOrganizationsHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListOrganizations {
                offset: 0,
                limit: 10,
            })
            .await
            .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total, 5);
    }

    #[tokio::test]
    async fn respects_offset_and_limit() {
        let mut mock = MockOrganizationReadStore::new();
        mock.expect_list()
            .withf(|offset, limit| *offset == 20 && *limit == 5)
            .returning(|_, _| Ok(vec![]));
        mock.expect_count().returning(|| Ok(25));

        let handler = ListOrganizationsHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListOrganizations {
                offset: 20,
                limit: 5,
            })
            .await
            .unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.total, 25);
        assert_eq!(result.offset, 20);
        assert_eq!(result.limit, 5);
    }
}
