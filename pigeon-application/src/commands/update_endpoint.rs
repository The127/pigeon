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
use crate::mediator::pipeline::RequestContext;
use crate::ports::stores::EventTypeReadStore;

#[derive(Debug)]
pub struct UpdateEndpoint {
    pub org_id: OrganizationId,
    pub id: EndpointId,
    pub url: String,
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
    event_type_read_store: Arc<dyn EventTypeReadStore>,
}

impl UpdateEndpointHandler {
    pub fn new(
        event_type_read_store: Arc<dyn EventTypeReadStore>,
    ) -> Self {
        Self { event_type_read_store }
    }
}

#[async_trait]
impl CommandHandler<UpdateEndpoint> for UpdateEndpointHandler {
    async fn handle(&self, command: UpdateEndpoint, ctx: &mut RequestContext) -> Result<Endpoint, ApplicationError> {
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


        let mut endpoint = ctx.uow()
            .endpoint_store()
            .find_by_id(&command.id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if endpoint.version() != command.version {
            return Err(ApplicationError::Conflict);
        }

        endpoint
            .update(command.url, command.event_type_ids)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        ctx.uow().endpoint_store().save(&endpoint).await?;
        ctx.uow().emit_event(DomainEvent::EndpointUpdated {
            endpoint_id: endpoint.id().clone(),
            app_id: endpoint.app_id().clone(),
            enabled: endpoint.enabled(),
        });

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
    use pigeon_domain::application::ApplicationId;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::organization::OrganizationId;

    fn empty_et_store(log: &OperationLog) -> Arc<dyn EventTypeReadStore> {
        Arc::new(FakeEventTypeReadStore::new(log.clone(), SharedEventTypeData::default()))
    }

    fn setup_with_endpoint() -> (OperationLog, Arc<FakeUnitOfWorkFactory>, Endpoint) {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let ep = Endpoint::new(
            ApplicationId::new(),
            None,
            "https://example.com/webhook".into(),
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

        let handler = UpdateEndpointHandler::new(empty_et_store(&log));
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id,
                url: "https://new.example.com/webhook".into(),
                event_type_ids: vec![],
                version,
            }, &mut ctx)
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
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_endpoint_does_not_exist() {
        let log = OperationLog::new();
        let et_store = empty_et_store(&log);
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = UpdateEndpointHandler::new(et_store);
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id: EndpointId::new(),
                url: "https://example.com/webhook".into(),
                event_type_ids: vec![],
                version: Version::new(0),
            }, &mut ctx)
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

        let handler = UpdateEndpointHandler::new(empty_et_store(&log));
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id,
                url: "".into(),
                event_type_ids: vec![],
                version,
            }, &mut ctx)
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

        let handler = UpdateEndpointHandler::new(empty_et_store(&log));
        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id,
                url: "https://example.com/webhook".into(),
                event_type_ids: vec![],
                version: Version::new(999),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Conflict)));
    }
}
