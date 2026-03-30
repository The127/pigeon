use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::organization::OrganizationId;

use async_trait::async_trait;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

#[derive(Debug)]
pub struct RevokeSigningSecret {
    pub org_id: OrganizationId,
    pub app_id: ApplicationId,
    pub endpoint_id: EndpointId,
    pub secret_index: usize,
}

impl Command for RevokeSigningSecret {
    type Output = ();

    fn command_name(&self) -> &'static str {
        "RevokeSigningSecret"
    }
}

#[derive(Default)]
pub struct RevokeSigningSecretHandler;

impl RevokeSigningSecretHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<RevokeSigningSecret> for RevokeSigningSecretHandler {
    async fn handle(
        &self,
        command: RevokeSigningSecret,
        ctx: &mut RequestContext,
    ) -> Result<(), ApplicationError> {
        let mut endpoint = ctx
            .uow()
            .endpoint_store()
            .find_by_id(&command.endpoint_id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        endpoint
            .revoke_signing_secret(command.secret_index)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        ctx.uow().endpoint_store().save(&endpoint).await?;

        Ok(())
    }
}
