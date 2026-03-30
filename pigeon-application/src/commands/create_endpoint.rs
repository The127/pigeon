use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::Endpoint;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;
use crate::ports::stores::EventTypeReadStore;

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
    event_type_read_store: Arc<dyn EventTypeReadStore>,
}

impl CreateEndpointHandler {
    pub fn new(
        event_type_read_store: Arc<dyn EventTypeReadStore>,
    ) -> Self {
        Self { event_type_read_store }
    }
}

#[async_trait]
impl CommandHandler<CreateEndpoint> for CreateEndpointHandler {
    async fn handle(&self, command: CreateEndpoint, ctx: &mut RequestContext) -> Result<Endpoint, ApplicationError> {
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

        ctx.uow().endpoint_store().insert(&endpoint).await?;

        Ok(endpoint)
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
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::organization::OrganizationId;

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
        let org_id = OrganizationId::new();
        let app_id = ApplicationId::new();
        let et = EventType::new(app_id.clone(), "user.created".into(), None).unwrap();
        let et_id = et.id().clone();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(read_store_with(&log, vec![et]));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), org_id.clone());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateEndpoint {
                org_id,
                app_id,
                name: None,
                url: "https://example.com/webhook".into(),
                signing_secret: Some("whsec_secret123".into()),
                event_type_ids: vec![et_id],
            }, &mut ctx)
            .await;

        let ep = result.unwrap();
        assert_eq!(ep.url(), "https://example.com/webhook");
    }

    #[tokio::test]
    async fn rejects_empty_url() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(empty_read_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateEndpoint {
                org_id: OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "".into(),
                signing_secret: Some("whsec_secret123".into()),
                event_type_ids: vec![],
            }, &mut ctx)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn creates_endpoint_without_signing_secret() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(empty_read_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateEndpoint {
                org_id: OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "https://example.com/webhook".into(),
                signing_secret: None,
                event_type_ids: vec![],
            }, &mut ctx)
            .await;

        let ep = result.unwrap();
        assert!(ep.signing_secret().is_none());
    }

    #[tokio::test]
    async fn rejects_nonexistent_event_type() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateEndpointHandler::new(empty_read_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateEndpoint {
                org_id: OrganizationId::new(),
                app_id: ApplicationId::new(),
                name: None,
                url: "https://example.com/webhook".into(),
                signing_secret: None,
                event_type_ids: vec![EventTypeId::new()],
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }
}
