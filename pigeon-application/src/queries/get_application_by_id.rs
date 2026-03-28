use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::ApplicationReadStore;

#[derive(Debug)]
pub struct GetApplicationById {
    pub org_id: OrganizationId,
    pub id: ApplicationId,
}

impl Query for GetApplicationById {
    type Output = Option<Application>;
}

pub struct GetApplicationByIdHandler {
    read_store: Arc<dyn ApplicationReadStore>,
}

impl GetApplicationByIdHandler {
    pub fn new(read_store: Arc<dyn ApplicationReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetApplicationById> for GetApplicationByIdHandler {
    async fn handle(
        &self,
        query: GetApplicationById,
    ) -> Result<Option<Application>, ApplicationError> {
        let app = self.read_store.find_by_id(&query.id).await?;
        match app {
            Some(ref a) if a.org_id() != &query.org_id => Ok(None),
            other => Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stores::MockApplicationReadStore;
    use pigeon_domain::application::ApplicationState;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn returns_application_when_found() {
        let app = Application::reconstitute(ApplicationState::fake());
        let expected_name = app.name().to_string();
        let org_id = app.org_id().clone();
        let id = app.id().clone();
        let query_id = app.id().clone();

        let mut mock = MockApplicationReadStore::new();
        mock.expect_find_by_id()
            .withf(move |arg_id| *arg_id == id)
            .returning(move |_| Ok(Some(app.clone())));

        let handler = GetApplicationByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetApplicationById { org_id, id: query_id })
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), expected_name);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let mut mock = MockApplicationReadStore::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let handler = GetApplicationByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetApplicationById {
                org_id: OrganizationId::new(),
                id: ApplicationId::new(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn returns_none_for_wrong_org() {
        let app = Application::reconstitute(ApplicationState::fake());
        let id = app.id().clone();

        let mut mock = MockApplicationReadStore::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(app.clone())));

        let handler = GetApplicationByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetApplicationById {
                org_id: OrganizationId::new(), // different org
                id,
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
