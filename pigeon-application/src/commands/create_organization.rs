use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::organization::Organization;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct CreateOrganization {
    pub name: String,
    pub slug: String,
}

impl Command for CreateOrganization {
    type Output = Organization;

    fn command_name(&self) -> &'static str {
        "CreateOrganization"
    }
}

pub struct CreateOrganizationHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl CreateOrganizationHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<CreateOrganization> for CreateOrganizationHandler {
    async fn handle(&self, command: CreateOrganization) -> Result<Organization, ApplicationError> {
        let org = Organization::new(command.name, command.slug)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let mut uow = self.uow_factory.begin().await?;
        uow.organization_store().insert(&org).await?;
        uow.commit().await?;

        Ok(org)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};

    #[tokio::test]
    async fn creates_organization_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(factory);

        let result = handler
            .handle(CreateOrganization {
                name: "my-org".into(),
                slug: "my-org".into(),
            })
            .await;

        let org = result.unwrap();
        assert_eq!(org.name(), "my-org");
        assert_eq!(org.slug(), "my-org");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "organization_store:insert",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(factory);

        let result = handler
            .handle(CreateOrganization {
                name: "".into(),
                slug: "my-org".into(),
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }

    #[tokio::test]
    async fn rejects_invalid_slug() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(factory);

        let result = handler
            .handle(CreateOrganization {
                name: "My Org".into(),
                slug: "My Org".into(),
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }
}
