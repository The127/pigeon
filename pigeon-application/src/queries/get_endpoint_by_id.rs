use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::endpoint::{Endpoint, EndpointId};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::EndpointReadStore;

#[derive(Debug)]
pub struct GetEndpointById {
    pub id: EndpointId,
    pub org_id: OrganizationId,
}

impl Query for GetEndpointById {
    type Output = Option<Endpoint>;
}

pub struct GetEndpointByIdHandler {
    read_store: Arc<dyn EndpointReadStore>,
}

impl GetEndpointByIdHandler {
    pub fn new(read_store: Arc<dyn EndpointReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetEndpointById> for GetEndpointByIdHandler {
    async fn handle(
        &self,
        query: GetEndpointById,
    ) -> Result<Option<Endpoint>, ApplicationError> {
        self.read_store.find_by_id(&query.id, &query.org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stores::MockEndpointReadStore;
    use pigeon_domain::endpoint::EndpointState;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn returns_endpoint_when_found() {
        let ep = Endpoint::reconstitute(EndpointState::fake());
        let expected_url = ep.url().to_string();
        let id = ep.id().clone();
        let query_id = ep.id().clone();

        let mut mock = MockEndpointReadStore::new();
        mock.expect_find_by_id()
            .withf(move |arg_id, _| *arg_id == id)
            .returning(move |_, _| Ok(Some(ep.clone())));

        let handler = GetEndpointByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetEndpointById { id: query_id, org_id: OrganizationId::new() })
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().url(), expected_url);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let mut mock = MockEndpointReadStore::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let handler = GetEndpointByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetEndpointById {
                id: EndpointId::new(),
                org_id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
