
use async_trait::async_trait;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

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

#[derive(Default)]
pub struct DeleteEndpointHandler;

impl DeleteEndpointHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<DeleteEndpoint> for DeleteEndpointHandler {
    async fn handle(&self, command: DeleteEndpoint, ctx: &mut RequestContext) -> Result<(), ApplicationError> {

        let existing = ctx.uow()
            .endpoint_store()
            .find_by_id(&command.id, &command.org_id)
            .await?;

        if existing.is_none() {
            return Err(ApplicationError::NotFound);
        }

        ctx.uow().endpoint_store().delete(&command.id).await?;

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
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::event_type::EventTypeId;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn deletes_endpoint_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let ep = Endpoint::new(
            ApplicationId::new(),
            None,
            "https://example.com/webhook".into(),
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

        let handler = DeleteEndpointHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(DeleteEndpoint { org_id: OrganizationId::new(), id }, &mut ctx)
            .await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "endpoint_store:find_by_id",
                "endpoint_store:delete",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_endpoint_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = DeleteEndpointHandler::new();
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(DeleteEndpoint {
                org_id: OrganizationId::new(),
                id: EndpointId::new(),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }
}
