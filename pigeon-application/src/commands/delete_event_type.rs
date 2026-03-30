
use async_trait::async_trait;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

#[derive(Debug)]
pub struct DeleteEventType {
    pub org_id: OrganizationId,
    pub id: EventTypeId,
}

impl Command for DeleteEventType {
    type Output = ();

    fn command_name(&self) -> &'static str {
        "DeleteEventType"
    }
}

#[derive(Default)]
pub struct DeleteEventTypeHandler;

impl DeleteEventTypeHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<DeleteEventType> for DeleteEventTypeHandler {
    async fn handle(&self, command: DeleteEventType, ctx: &mut RequestContext) -> Result<(), ApplicationError> {

        let existing = ctx.uow()
            .event_type_store()
            .find_by_id(&command.id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if existing.system() {
            return Err(ApplicationError::Validation(
                "System event types cannot be deleted".to_string(),
            ));
        }

        ctx.uow().event_type_store().delete(&command.id).await?;

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
    use pigeon_domain::application::ApplicationId;
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn deletes_event_type_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let et = EventType::new(ApplicationId::new(), "user.created".into(), None).unwrap();
        let id = et.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.event_type_store().insert(&et).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = DeleteEventTypeHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(DeleteEventType { org_id: OrganizationId::new(), id }, &mut ctx)
            .await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "event_type_store:find_by_id",
                "event_type_store:delete",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_event_type_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = DeleteEventTypeHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(DeleteEventType {
                org_id: OrganizationId::new(),
                id: EventTypeId::new(),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }
}
