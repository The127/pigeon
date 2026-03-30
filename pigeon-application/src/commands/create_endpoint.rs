use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::Endpoint;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::stores::EventTypeReadStore;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct CreateEndpoint {
    pub org_id: OrganizationId,
    pub app_id: ApplicationId,
    pub name: Option<String>,
    pub url: String,
    pub signing_secret: Option<String>,
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
    event_type_read_store: Arc<dyn EventTypeReadStore>,
}

impl CreateEndpointHandler {
    pub fn new(
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        event_type_read_store: Arc<dyn EventTypeReadStore>,
    ) -> Self {
        Self { uow_factory, event_type_read_store }
    }
}

#[async_trait]
impl CommandHandler<CreateEndpoint> for CreateEndpointHandler {
    async fn handle(&self, command: CreateEndpoint) -> Result<Endpoint, ApplicationError> {
        for et_id in &command.event_type_ids {
            if self.event_type_read_store
                .find_by_id(et_id, &command.org_id)
                .await?
                .is_none()
            {
                return Err(ApplicationError::Validation(
                    format!("Event type not found: {}", et_id.as_uuid()),
                ));
            }
        }

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
    use crate::test_support::fakes::{
        FakeEventTypeReadStore, FakeUnitOfWorkFactory, OperationLog, SharedEventTypeData,
    };
    use pigeon_domain::event_type::EventType;

    fn read_store_with(log: &OperationLog, event_types: Vec<EventType>) -> Arc<dyn EventTypeReadStore> {
        let data = SharedEventTypeData::default();
        data.event_types.lock().unwrap().extend(event_types);
        Arc::new(FakeEventTypeReadStore::new(log.clone(), data))
    }

    fn empty_read_store(log: &OperationLog) -> Arc<dyn EventTypeReadStore> {
        read_store_with(log, vec![])
    }

    #[tokio::test]
    async fn creates_endpoint_successfully() {
        let log = OperationLog::new();
        let org_id = pigeon_domain::organization::OrganizationId::new();
        let app_id = ApplicationId::new();
        let et = EventType::new(app_id.clone(), "user.created".into(), None).unwrap();
        let et_id = et.id().clone();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(factory, read_store_with(&log, vec![et]));

        let result = handler
            .handle(CreateEndpoint {
                org_id,
                app_id,
                name: None,
                url: "https://example.com/webhook".into(),
                signing_secret: Some("whsec_secret123".into()),
                event_type_ids: vec![et_id],
            })
            .await;

        let ep = result.unwrap();
        assert_eq!(ep.url(), "https://example.com/webhook");
    }

    #[tokio::test]
    async fn rejects_empty_url() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(factory, empty_read_store(&log));

        let result = handler
            .handle(CreateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "".into(),
                signing_secret: Some("whsec_secret123".into()),
                event_type_ids: vec![],
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn creates_endpoint_without_signing_secret() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(factory, empty_read_store(&log));

        let result = handler
            .handle(CreateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "https://example.com/webhook".into(),
                signing_secret: None,
                event_type_ids: vec![],
            })
            .await;

        let ep = result.unwrap();
        assert!(ep.signing_secret().is_none());
    }

    #[tokio::test]
    async fn rejects_nonexistent_event_type() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(factory, empty_read_store(&log));

        let result = handler
            .handle(CreateEndpoint {
                org_id: pigeon_domain::organization::OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "https://example.com/webhook".into(),
                signing_secret: None,
                event_type_ids: vec![EventTypeId::new()],
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }
}
