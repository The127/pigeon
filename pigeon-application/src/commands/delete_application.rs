use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct DeleteApplication {
    pub org_id: OrganizationId,
    pub id: ApplicationId,
}

impl Command for DeleteApplication {
    type Output = ();

    fn command_name(&self) -> &'static str {
        "DeleteApplication"
    }
}

pub struct DeleteApplicationHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl DeleteApplicationHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<DeleteApplication> for DeleteApplicationHandler {
    async fn handle(&self, command: DeleteApplication) -> Result<(), ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let existing = uow
            .application_store()
            .find_by_id(&command.id)
            .await?;

        match &existing {
            None => return Err(ApplicationError::NotFound),
            Some(app) if app.org_id() != &command.org_id => {
                return Err(ApplicationError::NotFound);
            }
            _ => {}
        }

        uow.application_store().delete(&command.id).await?;
        uow.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::application::Application;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn deletes_application_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let org_id = OrganizationId::new();
        let app = Application::new(org_id.clone(), "my-app".into(), "app_123".into()).unwrap();
        let id = app.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.application_store().insert(&app).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = DeleteApplicationHandler::new(factory);
        let result = handler
            .handle(DeleteApplication { org_id, id })
            .await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "application_store:find_by_id",
                "application_store:delete",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_application_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = DeleteApplicationHandler::new(factory);
        let result = handler
            .handle(DeleteApplication {
                org_id: OrganizationId::new(),
                id: ApplicationId::new(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn returns_not_found_for_wrong_org() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let org_id = OrganizationId::new();
        let app = Application::new(org_id, "my-app".into(), "app_123".into()).unwrap();
        let id = app.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.application_store().insert(&app).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = DeleteApplicationHandler::new(factory);
        let result = handler
            .handle(DeleteApplication {
                org_id: OrganizationId::new(), // different org
                id,
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }
}
