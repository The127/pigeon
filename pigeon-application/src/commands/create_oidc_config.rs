
use async_trait::async_trait;
use pigeon_domain::oidc_config::OidcConfig;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

#[derive(Debug)]
pub struct CreateOidcConfig {
    pub org_id: OrganizationId,
    pub issuer_url: String,
    pub audience: String,
    pub jwks_url: String,
}

impl Command for CreateOidcConfig {
    type Output = OidcConfig;

    fn command_name(&self) -> &'static str {
        "CreateOidcConfig"
    }
}

#[derive(Default)]
pub struct CreateOidcConfigHandler;

impl CreateOidcConfigHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<CreateOidcConfig> for CreateOidcConfigHandler {
    async fn handle(&self, command: CreateOidcConfig, ctx: &mut RequestContext) -> Result<OidcConfig, ApplicationError> {
        let config = OidcConfig::new(
            command.org_id,
            command.issuer_url,
            command.audience,
            command.jwks_url,
        )
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        ctx.uow().oidc_config_store().insert(&config).await?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;
    use crate::mediator::pipeline::RequestContext;
    use crate::ports::unit_of_work::UnitOfWorkFactory;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};

    #[tokio::test]
    async fn creates_oidc_config_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOidcConfigHandler::new();

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOidcConfig {
                org_id: OrganizationId::new(),
                issuer_url: "https://auth.example.com".into(),
                audience: "my-api".into(),
                jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await;

        let config = result.unwrap();
        assert_eq!(config.issuer_url(), "https://auth.example.com");
        assert_eq!(config.audience(), "my-api");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "oidc_config_store:insert",
            ]
        );
    }

    #[tokio::test]
    async fn rejects_empty_issuer_url() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOidcConfigHandler::new();

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOidcConfig {
                org_id: OrganizationId::new(),
                issuer_url: "".into(),
                audience: "my-api".into(),
                jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_empty_audience() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOidcConfigHandler::new();

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOidcConfig {
                org_id: OrganizationId::new(),
                issuer_url: "https://auth.example.com".into(),
                audience: "".into(),
                jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_empty_jwks_url() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOidcConfigHandler::new();

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOidcConfig {
                org_id: OrganizationId::new(),
                issuer_url: "https://auth.example.com".into(),
                audience: "my-api".into(),
                jwks_url: "".into(),
            }, &mut ctx)
            .await;

        assert!(result.is_err());
    }
}
