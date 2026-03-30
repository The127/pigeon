
use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::event::DomainEvent;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

/// Internal command used by sagas — not exposed via API.
/// Disables an endpoint by app_id + endpoint_id (no org_id needed).
#[derive(Debug)]
pub struct DisableEndpoint {
    pub app_id: ApplicationId,
    pub endpoint_id: EndpointId,
}

impl Command for DisableEndpoint {
    type Output = ();

    fn command_name(&self) -> &'static str {
        "DisableEndpoint"
    }
}

#[derive(Default)]
pub struct DisableEndpointHandler;

impl DisableEndpointHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<DisableEndpoint> for DisableEndpointHandler {
    async fn handle(&self, command: DisableEndpoint, ctx: &mut RequestContext) -> Result<(), ApplicationError> {

        let mut endpoint = ctx.uow()
            .endpoint_store()
            .find_by_app_and_id(&command.endpoint_id, &command.app_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if !endpoint.enabled() {
            return Ok(()); // already disabled
        }

        endpoint.disable();
        ctx.uow().endpoint_store().save(&endpoint).await?;
        ctx.uow().emit_event(DomainEvent::EndpointUpdated {
            endpoint_id: endpoint.id().clone(),
            app_id: endpoint.app_id().clone(),
            enabled: false,
        });

        Ok(())
    }
}
