
use async_trait::async_trait;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

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

#[derive(Default)]
pub struct DeleteOrganizationHandler;

impl DeleteOrganizationHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<DeleteOrganization> for DeleteOrganizationHandler {
    async fn handle(&self, command: DeleteOrganization, ctx: &mut RequestContext) -> Result<(), ApplicationError> {

        let existing = ctx.uow()
            .organization_store()
            .find_by_id(&command.id)
            .await?;

        if existing.is_none() {
            return Err(ApplicationError::NotFound);
        }

        ctx.uow().organization_store().delete(&command.id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;
    use crate::mediator::pipeline::RequestContext;
    use crate::ports::unit_of_work::UnitOfWorkFactory;
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

        let handler = DeleteOrganizationHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(DeleteOrganization { id }, &mut ctx)
            .await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "organization_store:find_by_id",
                "organization_store:delete",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_organization_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = DeleteOrganizationHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(DeleteOrganization {
                id: OrganizationId::new(),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }
}
