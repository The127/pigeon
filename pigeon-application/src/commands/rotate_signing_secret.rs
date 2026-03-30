use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::{Endpoint, EndpointId};
use pigeon_domain::organization::OrganizationId;

use async_trait::async_trait;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

#[derive(Debug)]
pub struct RotateSigningSecret {
    pub org_id: OrganizationId,
    pub app_id: ApplicationId,
    pub endpoint_id: EndpointId,
}

impl Command for RotateSigningSecret {
    type Output = RotateSigningSecretResult;

    fn command_name(&self) -> &'static str {
        "RotateSigningSecret"
    }
}

#[derive(Debug)]
pub struct RotateSigningSecretResult {
    pub endpoint: Endpoint,
    /// The full new secret — returned once so the user can copy it.
    pub new_secret: String,
}

#[derive(Default)]
pub struct RotateSigningSecretHandler;

impl RotateSigningSecretHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<RotateSigningSecret> for RotateSigningSecretHandler {
    async fn handle(
        &self,
        command: RotateSigningSecret,
        ctx: &mut RequestContext,
    ) -> Result<RotateSigningSecretResult, ApplicationError> {
        let mut endpoint = ctx
            .uow()
            .endpoint_store()
            .find_by_id(&command.endpoint_id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        let new_secret = endpoint
            .rotate_signing_secret()
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        ctx.uow().endpoint_store().save(&endpoint).await?;

        Ok(RotateSigningSecretResult {
            endpoint,
            new_secret,
        })
    }
}
