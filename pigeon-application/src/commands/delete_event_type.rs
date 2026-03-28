use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

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

pub struct DeleteEventTypeHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl DeleteEventTypeHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<DeleteEventType> for DeleteEventTypeHandler {
    async fn handle(&self, command: DeleteEventType) -> Result<(), ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let existing = uow
            .event_type_store()
            .find_by_id(&command.id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if existing.system() {
            return Err(ApplicationError::Validation(
                "System event types cannot be deleted".to_string(),
            ));
        }

        uow.event_type_store().delete(&command.id).await?;
        uow.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::application::ApplicationId;
    use pigeon_domain::event_type::EventType;

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

        let handler = DeleteEventTypeHandler::new(factory);
        let result = handler
            .handle(DeleteEventType { org_id: pigeon_domain::organization::OrganizationId::new(), id })
            .await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "event_type_store:find_by_id",
                "event_type_store:delete",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_event_type_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = DeleteEventTypeHandler::new(factory);
        let result = handler
            .handle(DeleteEventType {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                id: EventTypeId::new(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }
}
