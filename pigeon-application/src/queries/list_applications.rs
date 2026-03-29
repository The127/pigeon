use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::Application;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::ApplicationReadStore;
use crate::queries::PaginatedResult;

#[derive(Debug)]
pub struct ListApplications {
    pub org_id: OrganizationId,
    pub search: Option<String>,
    pub offset: u64,
    pub limit: u64,
}

impl Query for ListApplications {
    type Output = PaginatedResult<Application>;
}

pub struct ListApplicationsHandler {
    read_store: Arc<dyn ApplicationReadStore>,
}

impl ListApplicationsHandler {
    pub fn new(read_store: Arc<dyn ApplicationReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<ListApplications> for ListApplicationsHandler {
    async fn handle(
        &self,
        query: ListApplications,
    ) -> Result<PaginatedResult<Application>, ApplicationError> {
        let items = self.read_store.list_by_org(&query.org_id, query.search.clone(), query.offset, query.limit).await?;
        let total = self.read_store.count_by_org(&query.org_id, query.search).await?;

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
    use crate::ports::stores::MockApplicationReadStore;
    use pigeon_domain::application::ApplicationState;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn returns_empty_list() {
        let mut mock = MockApplicationReadStore::new();
        mock.expect_list_by_org().returning(|_, _, _, _| Ok(vec![]));
        mock.expect_count_by_org().returning(|_, _| Ok(0));

        let handler = ListApplicationsHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListApplications {
                org_id: OrganizationId::new(),
                search: None,
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
        let app1 = Application::reconstitute(ApplicationState::fake());
        let app2 = Application::reconstitute(ApplicationState::fake());
        let items = vec![app1, app2];
        let items_clone = items.clone();

        let mut mock = MockApplicationReadStore::new();
        mock.expect_list_by_org()
            .withf(|_, _, offset, limit| *offset == 0 && *limit == 10)
            .returning(move |_, _, _, _| Ok(items_clone.clone()));
        mock.expect_count_by_org().returning(|_, _| Ok(5));

        let handler = ListApplicationsHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListApplications {
                org_id: OrganizationId::new(),
                search: None,
                offset: 0,
                limit: 10,
            })
            .await
            .unwrap();

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total, 5);
    }

    #[tokio::test]
    async fn lists_only_apps_for_requested_org() {
        let org_a = OrganizationId::new();
        let org_b = OrganizationId::new();

        let mut state_a = ApplicationState::fake();
        state_a.org_id = org_a.clone();
        let app_a = Application::reconstitute(state_a);

        let mut state_b = ApplicationState::fake();
        state_b.org_id = org_b.clone();
        let app_b = Application::reconstitute(state_b);

        let items_a = vec![app_a.clone()];
        let items_a_clone = items_a.clone();
        let org_a_clone = org_a.clone();

        let mut mock = MockApplicationReadStore::new();
        mock.expect_list_by_org()
            .withf(move |org_id, _, _, _| *org_id == org_a_clone)
            .returning(move |_, _, _, _| Ok(items_a_clone.clone()));
        mock.expect_count_by_org()
            .returning(|_, _| Ok(1));

        let handler = ListApplicationsHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListApplications {
                org_id: org_a,
                search: None,
                offset: 0,
                limit: 10,
            })
            .await
            .unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].name(), app_a.name());
    }

    #[tokio::test]
    async fn respects_offset_and_limit() {
        let mut mock = MockApplicationReadStore::new();
        mock.expect_list_by_org()
            .withf(|_, _, offset, limit| *offset == 20 && *limit == 5)
            .returning(|_, _, _, _| Ok(vec![]));
        mock.expect_count_by_org().returning(|_, _| Ok(25));

        let handler = ListApplicationsHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListApplications {
                org_id: OrganizationId::new(),
                search: None,
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
