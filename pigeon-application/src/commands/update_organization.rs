
use async_trait::async_trait;
use pigeon_domain::organization::{Organization, OrganizationId};
use pigeon_domain::version::Version;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

#[derive(Debug)]
pub struct UpdateOrganization {
    pub id: OrganizationId,
    pub name: String,
    pub version: Version,
}

impl Command for UpdateOrganization {
    type Output = Organization;

    fn command_name(&self) -> &'static str {
        "UpdateOrganization"
    }
}

#[derive(Default)]
pub struct UpdateOrganizationHandler;

impl UpdateOrganizationHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<UpdateOrganization> for UpdateOrganizationHandler {
    async fn handle(&self, command: UpdateOrganization, ctx: &mut RequestContext) -> Result<Organization, ApplicationError> {

        let mut org = ctx.uow()
            .organization_store()
            .find_by_id(&command.id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if org.version() != command.version {
            return Err(ApplicationError::Conflict);
        }

        org.rename(command.name)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        ctx.uow().organization_store().save(&org).await?;

        Ok(org)
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

    fn setup_with_org() -> (OperationLog, Arc<FakeUnitOfWorkFactory>, Organization) {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let org = Organization::new("original-name".into(), "original-slug".into()).unwrap();
        (log, factory, org)
    }

    #[tokio::test]
    async fn updates_organization_successfully() {
        let (log, factory, org) = setup_with_org();
        let id = org.id().clone();
        let version = org.version();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.organization_store().insert(&org).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateOrganizationHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateOrganization {
                id,
                name: "new-name".into(),
                version,
            }, &mut ctx)
            .await;

        let updated = result.unwrap();
        assert_eq!(updated.name(), "new-name");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "organization_store:find_by_id",
                "organization_store:save",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_organization_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = UpdateOrganizationHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateOrganization {
                id: OrganizationId::new(),
                name: "new-name".into(),
                version: Version::new(0),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let (log, factory, org) = setup_with_org();
        let id = org.id().clone();
        let version = org.version();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.organization_store().insert(&org).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateOrganizationHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateOrganization {
                id,
                name: "".into(),
                version,
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }

    #[tokio::test]
    async fn rejects_version_conflict() {
        let (log, factory, org) = setup_with_org();
        let id = org.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.organization_store().insert(&org).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateOrganizationHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateOrganization {
                id,
                name: "new-name".into(),
                version: Version::new(999),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Conflict)));
    }
}
