use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct DeleteOrganization {
    pub id: OrganizationId,
}

impl Command for DeleteOrganization {
    type Output = ();

    fn command_name(&self) -> &'static str {
        "DeleteOrganization"
    }
}

pub struct DeleteOrganizationHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl DeleteOrganizationHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<DeleteOrganization> for DeleteOrganizationHandler {
    async fn handle(&self, command: DeleteOrganization) -> Result<(), ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let existing = uow
            .organization_store()
            .find_by_id(&command.id)
            .await?;

        if existing.is_none() {
            return Err(ApplicationError::NotFound);
        }

        uow.organization_store().delete(&command.id).await?;
        uow.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::organization::Organization;

    #[tokio::test]
    async fn deletes_organization_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let org = Organization::new("my-org".into(), "my-org".into()).unwrap();
        let id = org.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.organization_store().insert(&org).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = DeleteOrganizationHandler::new(factory);
        let result = handler
            .handle(DeleteOrganization { id })
            .await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "organization_store:find_by_id",
                "organization_store:delete",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_organization_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = DeleteOrganizationHandler::new(factory);
        let result = handler
            .handle(DeleteOrganization {
                id: OrganizationId::new(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }
}
