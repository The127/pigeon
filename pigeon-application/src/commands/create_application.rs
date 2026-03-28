use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::Application;
use pigeon_domain::event_type::{EventType, TEST_EVENT_TYPE_NAME};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct CreateApplication {
    pub org_id: OrganizationId,
    pub name: String,
    pub uid: String,
}

impl Command for CreateApplication {
    type Output = Application;

    fn command_name(&self) -> &'static str {
        "CreateApplication"
    }
}

pub struct CreateApplicationHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl CreateApplicationHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<CreateApplication> for CreateApplicationHandler {
    async fn handle(&self, command: CreateApplication) -> Result<Application, ApplicationError> {
        let app = Application::new(command.org_id, command.name, command.uid)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let test_event_type =
            EventType::new_system(app.id().clone(), TEST_EVENT_TYPE_NAME.to_string());

        let mut uow = self.uow_factory.begin().await?;
        uow.application_store().insert(&app).await?;
        uow.event_type_store().insert(&test_event_type).await?;
        uow.commit().await?;

        Ok(app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn creates_application_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateApplicationHandler::new(factory);
        let org_id = OrganizationId::new();

        let result = handler
            .handle(CreateApplication {
                org_id: org_id.clone(),
                name: "my-app".into(),
                uid: "app_123".into(),
            })
            .await;

        let app = result.unwrap();
        assert_eq!(app.name(), "my-app");
        assert_eq!(app.uid(), "app_123");
        assert_eq!(app.org_id(), &org_id);
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "application_store:insert",
                "event_type_store:insert",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateApplicationHandler::new(factory);

        let result = handler
            .handle(CreateApplication {
                org_id: OrganizationId::new(),
                name: "".into(),
                uid: "app_123".into(),
            })
            .await;

        assert!(result.is_err());
        // UoW should never be started for a validation failure
        assert!(log.entries().is_empty());
    }
}
