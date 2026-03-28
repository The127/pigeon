use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::organization::{Organization, OrganizationId};

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::OrganizationReadStore;

#[derive(Debug)]
pub struct GetOrganizationById {
    pub id: OrganizationId,
}

impl Query for GetOrganizationById {
    type Output = Option<Organization>;
}

pub struct GetOrganizationByIdHandler {
    read_store: Arc<dyn OrganizationReadStore>,
}

impl GetOrganizationByIdHandler {
    pub fn new(read_store: Arc<dyn OrganizationReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetOrganizationById> for GetOrganizationByIdHandler {
    async fn handle(
        &self,
        query: GetOrganizationById,
    ) -> Result<Option<Organization>, ApplicationError> {
        self.read_store.find_by_id(&query.id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stores::MockOrganizationReadStore;
    use pigeon_domain::organization::OrganizationState;

    #[tokio::test]
    async fn returns_organization_when_found() {
        let org = Organization::reconstitute(OrganizationState::fake());
        let expected_name = org.name().to_string();
        let id = org.id().clone();
        let query_id = org.id().clone();

        let mut mock = MockOrganizationReadStore::new();
        mock.expect_find_by_id()
            .withf(move |arg_id| *arg_id == id)
            .returning(move |_| Ok(Some(org.clone())));

        let handler = GetOrganizationByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetOrganizationById { id: query_id })
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), expected_name);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let mut mock = MockOrganizationReadStore::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let handler = GetOrganizationByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetOrganizationById {
                id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
