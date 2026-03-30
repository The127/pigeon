
use async_trait::async_trait;
use pigeon_domain::event_type::{EventType, EventTypeId};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use serde_json::Value;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

#[derive(Debug)]
pub struct UpdateEventType {
    pub org_id: OrganizationId,
    pub id: EventTypeId,
    pub name: String,
    pub schema: Option<Value>,
    pub version: Version,
}

impl Command for UpdateEventType {
    type Output = EventType;

    fn command_name(&self) -> &'static str {
        "UpdateEventType"
    }
}

#[derive(Default)]
pub struct UpdateEventTypeHandler;

impl UpdateEventTypeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<UpdateEventType> for UpdateEventTypeHandler {
    async fn handle(&self, command: UpdateEventType, ctx: &mut RequestContext) -> Result<EventType, ApplicationError> {

        let mut event_type = ctx.uow()
            .event_type_store()
            .find_by_id(&command.id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if event_type.version() != command.version {
            return Err(ApplicationError::Conflict);
        }

        event_type
            .update(command.name, command.schema)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        ctx.uow().event_type_store().save(&event_type).await?;

        Ok(event_type)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;
    use crate::mediator::pipeline::RequestContext;
    use crate::ports::unit_of_work::UnitOfWorkFactory;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::application::ApplicationId;
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::organization::OrganizationId;

    fn setup_with_event_type() -> (OperationLog, Arc<FakeUnitOfWorkFactory>, EventType) {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let et = EventType::new(ApplicationId::new(), "original.event".into(), None).unwrap();
        (log, factory, et)
    }

    #[tokio::test]
    async fn updates_event_type_successfully() {
        let (log, factory, et) = setup_with_event_type();
        let id = et.id().clone();
        let version = et.version();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.event_type_store().insert(&et).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateEventTypeHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateEventType {
                org_id: OrganizationId::new(),
                id,
                name: "new.event".into(),
                schema: None,
                version,
            }, &mut ctx)
            .await;

        let updated = result.unwrap();
        assert_eq!(updated.name(), "new.event");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "event_type_store:find_by_id",
                "event_type_store:save",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_event_type_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = UpdateEventTypeHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateEventType {
                org_id: OrganizationId::new(),
                id: EventTypeId::new(),
                name: "new.event".into(),
                schema: None,
                version: Version::new(0),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let (log, factory, et) = setup_with_event_type();
        let id = et.id().clone();
        let version = et.version();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.event_type_store().insert(&et).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateEventTypeHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateEventType {
                org_id: OrganizationId::new(),
                id,
                name: "".into(),
                schema: None,
                version,
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }

    #[tokio::test]
    async fn rejects_version_conflict() {
        let (log, factory, et) = setup_with_event_type();
        let id = et.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.event_type_store().insert(&et).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateEventTypeHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateEventType {
                org_id: OrganizationId::new(),
                id,
                name: "new.event".into(),
                schema: None,
                version: Version::new(999),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Conflict)));
    }
}
