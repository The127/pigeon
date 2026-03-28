use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::oidc_config::OidcConfig;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::OidcConfigReadStore;
use crate::queries::PaginatedResult;

#[derive(Debug)]
pub struct ListOidcConfigsByOrg {
    pub org_id: OrganizationId,
    pub offset: u64,
    pub limit: u64,
}

impl Query for ListOidcConfigsByOrg {
    type Output = PaginatedResult<OidcConfig>;
}

pub struct ListOidcConfigsByOrgHandler {
    read_store: Arc<dyn OidcConfigReadStore>,
}

impl ListOidcConfigsByOrgHandler {
    pub fn new(read_store: Arc<dyn OidcConfigReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<ListOidcConfigsByOrg> for ListOidcConfigsByOrgHandler {
    async fn handle(
        &self,
        query: ListOidcConfigsByOrg,
    ) -> Result<PaginatedResult<OidcConfig>, ApplicationError> {
        let items = self
            .read_store
            .list_by_org(&query.org_id, query.offset, query.limit)
            .await?;
        let total = self.read_store.count_by_org(&query.org_id).await?;

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
    use crate::ports::stores::MockOidcConfigReadStore;
    use pigeon_domain::oidc_config::OidcConfigState;

    #[tokio::test]
    async fn returns_empty_list() {
        let org_id = OrganizationId::new();
        let org_id_clone = org_id.clone();

        let mut mock = MockOidcConfigReadStore::new();
        mock.expect_list_by_org().returning(|_, _, _| Ok(vec![]));
        mock.expect_count_by_org().returning(|_| Ok(0));

        let handler = ListOidcConfigsByOrgHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListOidcConfigsByOrg {
                org_id: org_id_clone,
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
        let config1 = OidcConfig::reconstitute(OidcConfigState::fake());
        let config2 = OidcConfig::reconstitute(OidcConfigState::fake());
        let items = vec![config1, config2];
        let items_clone = items.clone();

        let mut mock = MockOidcConfigReadStore::new();
        mock.expect_list_by_org()
            .withf(|_, offset, limit| *offset == 0 && *limit == 10)
            .returning(move |_, _, _| Ok(items_clone.clone()));
        mock.expect_count_by_org().returning(|_| Ok(5));

        let handler = ListOidcConfigsByOrgHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListOidcConfigsByOrg {
                org_id: OrganizationId::new(),
                offset: 0,
                limit: 10,
            })
            .await
            .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total, 5);
    }
}
