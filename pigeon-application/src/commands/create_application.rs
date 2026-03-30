use async_trait::async_trait;
use pigeon_domain::application::Application;
use pigeon_domain::event_type::{EventType, TEST_EVENT_TYPE_NAME};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

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

#[derive(Default)]
pub struct CreateApplicationHandler;

impl CreateApplicationHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<CreateApplication> for CreateApplicationHandler {
    async fn handle(&self, command: CreateApplication, ctx: &mut RequestContext) -> Result<Application, ApplicationError> {
        let app = Application::new(command.org_id, command.name, command.uid)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let test_event_type =
            EventType::new_system(app.id().clone(), TEST_EVENT_TYPE_NAME.to_string());

        ctx.uow().application_store().insert(&app).await?;
        ctx.uow().event_type_store().insert(&test_event_type).await?;

        Ok(app)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;
    use crate::mediator::pipeline::RequestContext;
    use crate::ports::unit_of_work::UnitOfWorkFactory;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn creates_application_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateApplicationHandler::new();
        let org_id = OrganizationId::new();

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), org_id.clone());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateApplication {
                org_id: org_id.clone(),
                name: "my-app".into(),
                uid: "app_123".into(),
            }, &mut ctx)
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
            ]
        );
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateApplicationHandler::new();

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateApplication {
                org_id: OrganizationId::new(),
                name: "".into(),
                uid: "app_123".into(),
            }, &mut ctx)
            .await;

        assert!(result.is_err());
    }
}
