use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventType;
use pigeon_domain::organization::OrganizationId;
use serde_json::Value;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;
use crate::ports::stores::EventTypeReadStore;

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
    event_type_read_store: Arc<dyn EventTypeReadStore>,
}

impl CreateEventTypeHandler {
    pub fn new(
        event_type_read_store: Arc<dyn EventTypeReadStore>,
    ) -> Self {
        Self { event_type_read_store }
    }
}

#[async_trait]
impl CommandHandler<CreateEventType> for CreateEventTypeHandler {
    async fn handle(&self, command: CreateEventType, ctx: &mut RequestContext) -> Result<EventType, ApplicationError> {
        let event_type = EventType::new(command.app_id, command.name, command.schema)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let existing = self.event_type_read_store
            .find_by_app_and_name(event_type.app_id(), event_type.name(), &command.org_id)
            .await?;
        if existing.is_some() {
            return Err(ApplicationError::Validation(
                "Event type with this name already exists for this application".to_string(),
            ));
        }

        ctx.uow().event_type_store().insert(&event_type).await?;

        Ok(event_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mediator::pipeline::RequestContext;
    use crate::ports::unit_of_work::UnitOfWorkFactory;
    use crate::test_support::fakes::{
        FakeEventTypeReadStore, FakeUnitOfWorkFactory, OperationLog, SharedEventTypeData,
    };

    fn empty_read_store(log: &OperationLog) -> Arc<dyn EventTypeReadStore> {
        Arc::new(FakeEventTypeReadStore::new(log.clone(), SharedEventTypeData::default()))
    }

    #[tokio::test]
    async fn creates_event_type_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEventTypeHandler::new(empty_read_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateEventType {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: "user.created".into(),
                schema: None,
            }, &mut ctx)
            .await;

        let et = result.unwrap();
        assert_eq!(et.name(), "user.created");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "event_type_read_store:find_by_app_and_name",
                "event_type_store:insert",
            ]
        );
    }

    #[tokio::test]
    async fn rejects_duplicate_name() {
        let log = OperationLog::new();
        let app_id = ApplicationId::new();
        let org_id = pigeon_domain::organization::OrganizationId::new();
        let existing = EventType::new(app_id.clone(), "user.created".into(), None).unwrap();
        let et_data = SharedEventTypeData::default();
        et_data.event_types.lock().unwrap().push(existing);
        let read_store = Arc::new(FakeEventTypeReadStore::new(log.clone(), et_data));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEventTypeHandler::new(read_store);

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), org_id.clone());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateEventType {
                org_id,
                app_id,
                name: "user.created".into(),
                schema: None,
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEventTypeHandler::new(empty_read_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateEventType {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: "".into(),
                schema: None,
            }, &mut ctx)
            .await;

        assert!(result.is_err());
    }
}
