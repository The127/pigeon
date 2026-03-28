use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventType;
use pigeon_domain::organization::OrganizationId;
use serde_json::Value;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct CreateEventType {
    pub org_id: OrganizationId,
    pub app_id: ApplicationId,
    pub name: String,
    pub schema: Option<Value>,
}

impl Command for CreateEventType {
    type Output = EventType;

    fn command_name(&self) -> &'static str {
        "CreateEventType"
    }
}

pub struct CreateEventTypeHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl CreateEventTypeHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<CreateEventType> for CreateEventTypeHandler {
    async fn handle(&self, command: CreateEventType) -> Result<EventType, ApplicationError> {
        let event_type = EventType::new(command.app_id, command.name, command.schema)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let mut uow = self.uow_factory.begin().await?;
        uow.event_type_store().insert(&event_type).await?;
        uow.commit().await?;

        Ok(event_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};

    #[tokio::test]
    async fn creates_event_type_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEventTypeHandler::new(factory);

        let result = handler
            .handle(CreateEventType {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: "user.created".into(),
                schema: None,
            })
            .await;

        let et = result.unwrap();
        assert_eq!(et.name(), "user.created");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "event_type_store:insert",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEventTypeHandler::new(factory);

        let result = handler
            .handle(CreateEventType {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: "".into(),
                schema: None,
            })
            .await;

        assert!(result.is_err());
        // UoW should never be started for a validation failure
        assert!(log.entries().is_empty());
    }
}
