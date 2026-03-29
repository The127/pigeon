use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::Endpoint;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct CreateEndpoint {
    pub org_id: OrganizationId,
    pub app_id: ApplicationId,
    pub name: Option<String>,
    pub url: String,
    pub signing_secret: String,
    pub event_type_ids: Vec<EventTypeId>,
}

impl Command for CreateEndpoint {
    type Output = Endpoint;

    fn command_name(&self) -> &'static str {
        "CreateEndpoint"
    }
}

pub struct CreateEndpointHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl CreateEndpointHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<CreateEndpoint> for CreateEndpointHandler {
    async fn handle(&self, command: CreateEndpoint) -> Result<Endpoint, ApplicationError> {
        let endpoint = Endpoint::new(
            command.app_id,
            command.name,
            command.url,
            command.signing_secret,
            command.event_type_ids,
        )
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let mut uow = self.uow_factory.begin().await?;
        uow.endpoint_store().insert(&endpoint).await?;
        uow.commit().await?;

        Ok(endpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};

    #[tokio::test]
    async fn creates_endpoint_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(factory);

        let result = handler
            .handle(CreateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "https://example.com/webhook".into(),
                signing_secret: "whsec_secret123".into(),
                event_type_ids: vec![EventTypeId::new()],
            })
            .await;

        let ep = result.unwrap();
        assert_eq!(ep.url(), "https://example.com/webhook");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "endpoint_store:insert",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn rejects_empty_url() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(factory);

        let result = handler
            .handle(CreateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "".into(),
                signing_secret: "whsec_secret123".into(),
                event_type_ids: vec![],
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }

    #[tokio::test]
    async fn rejects_empty_signing_secret() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(factory);

        let result = handler
            .handle(CreateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "https://example.com/webhook".into(),
                signing_secret: "".into(),
                event_type_ids: vec![],
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }
}
