use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct DeleteEndpoint {
    pub org_id: OrganizationId,
    pub id: EndpointId,
}

impl Command for DeleteEndpoint {
    type Output = ();

    fn command_name(&self) -> &'static str {
        "DeleteEndpoint"
    }
}

pub struct DeleteEndpointHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl DeleteEndpointHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<DeleteEndpoint> for DeleteEndpointHandler {
    async fn handle(&self, command: DeleteEndpoint) -> Result<(), ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let existing = uow
            .endpoint_store()
            .find_by_id(&command.id, &command.org_id)
            .await?;

        if existing.is_none() {
            return Err(ApplicationError::NotFound);
        }

        uow.endpoint_store().delete(&command.id).await?;
        uow.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::application::ApplicationId;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::event_type::EventTypeId;

    #[tokio::test]
    async fn deletes_endpoint_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let ep = Endpoint::new(
            ApplicationId::new(),
            None,
            "https://example.com/webhook".into(),
            "whsec_secret123".into(),
            vec![EventTypeId::new()],
        )
        .unwrap();
        let id = ep.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.endpoint_store().insert(&ep).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = DeleteEndpointHandler::new(factory);
        let result = handler
            .handle(DeleteEndpoint { org_id: pigeon_domain::organization::OrganizationId::new(), id })
            .await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "endpoint_store:find_by_id",
                "endpoint_store:delete",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_endpoint_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = DeleteEndpointHandler::new(factory);
        let result = handler
            .handle(DeleteEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                id: EndpointId::new(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }
}
