use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::endpoint::{Endpoint, EndpointId};
use pigeon_domain::event::DomainEvent;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct UpdateEndpoint {
    pub org_id: OrganizationId,
    pub id: EndpointId,
    pub url: String,
    pub signing_secret: String,
    pub event_type_ids: Vec<EventTypeId>,
    pub version: Version,
}

impl Command for UpdateEndpoint {
    type Output = Endpoint;

    fn command_name(&self) -> &'static str {
        "UpdateEndpoint"
    }
}

pub struct UpdateEndpointHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl UpdateEndpointHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<UpdateEndpoint> for UpdateEndpointHandler {
    async fn handle(&self, command: UpdateEndpoint) -> Result<Endpoint, ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let mut endpoint = uow
            .endpoint_store()
            .find_by_id(&command.id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if endpoint.version() != command.version {
            return Err(ApplicationError::Conflict);
        }

        endpoint
            .update(command.url, command.signing_secret, command.event_type_ids)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        uow.endpoint_store().save(&endpoint).await?;
        uow.emit_event(DomainEvent::EndpointUpdated {
            endpoint_id: endpoint.id().clone(),
            app_id: endpoint.app_id().clone(),
            enabled: endpoint.enabled(),
        });
        uow.commit().await?;

        Ok(endpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::application::ApplicationId;
    use pigeon_domain::endpoint::Endpoint;

    fn setup_with_endpoint() -> (OperationLog, Arc<FakeUnitOfWorkFactory>, Endpoint) {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let ep = Endpoint::new(
            ApplicationId::new(),
            "https://example.com/webhook".into(),
            "whsec_secret123".into(),
            vec![EventTypeId::new()],
        )
        .unwrap();
        (log, factory, ep)
    }

    #[tokio::test]
    async fn updates_endpoint_successfully() {
        let (log, factory, ep) = setup_with_endpoint();
        let id = ep.id().clone();
        let version = ep.version();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.endpoint_store().insert(&ep).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateEndpointHandler::new(factory);
        let result = handler
            .handle(UpdateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                id,
                url: "https://new.example.com/webhook".into(),
                signing_secret: "whsec_new_secret".into(),
                event_type_ids: vec![],
                version,
            })
            .await;

        let updated = result.unwrap();
        assert_eq!(updated.url(), "https://new.example.com/webhook");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "endpoint_store:find_by_id",
                "endpoint_store:save",
                "uow:emit_event:endpoint_updated",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_endpoint_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = UpdateEndpointHandler::new(factory);
        let result = handler
            .handle(UpdateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                id: EndpointId::new(),
                url: "https://example.com/webhook".into(),
                signing_secret: "whsec_secret".into(),
                event_type_ids: vec![],
                version: Version::new(0),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn rejects_empty_url() {
        let (log, factory, ep) = setup_with_endpoint();
        let id = ep.id().clone();
        let version = ep.version();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.endpoint_store().insert(&ep).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateEndpointHandler::new(factory);
        let result = handler
            .handle(UpdateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                id,
                url: "".into(),
                signing_secret: "whsec_secret".into(),
                event_type_ids: vec![],
                version,
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }

    #[tokio::test]
    async fn rejects_version_conflict() {
        let (log, factory, ep) = setup_with_endpoint();
        let id = ep.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.endpoint_store().insert(&ep).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateEndpointHandler::new(factory);
        let result = handler
            .handle(UpdateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                id,
                url: "https://example.com/webhook".into(),
                signing_secret: "whsec_secret".into(),
                event_type_ids: vec![],
                version: Version::new(999),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::Conflict)));
    }
}
