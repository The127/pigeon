use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::EndpointId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

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

pub struct DisableEndpointHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl DisableEndpointHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<DisableEndpoint> for DisableEndpointHandler {
    async fn handle(&self, command: DisableEndpoint) -> Result<(), ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let mut endpoint = uow
            .endpoint_store()
            .find_by_app_and_id(&command.endpoint_id, &command.app_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if !endpoint.enabled() {
            return Ok(()); // already disabled
        }

        endpoint.disable();
        uow.endpoint_store().save(&endpoint).await?;
        uow.commit().await?;

        Ok(())
    }
}
