use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::oidc_config::OidcConfig;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

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

pub struct CreateOidcConfigHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl CreateOidcConfigHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<CreateOidcConfig> for CreateOidcConfigHandler {
    async fn handle(&self, command: CreateOidcConfig) -> Result<OidcConfig, ApplicationError> {
        let config = OidcConfig::new(
            command.org_id,
            command.issuer_url,
            command.audience,
            command.jwks_url,
        )
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let mut uow = self.uow_factory.begin().await?;
        uow.oidc_config_store().insert(&config).await?;
        uow.commit().await?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};

    #[tokio::test]
    async fn creates_oidc_config_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOidcConfigHandler::new(factory);

        let result = handler
            .handle(CreateOidcConfig {
                org_id: OrganizationId::new(),
                issuer_url: "https://auth.example.com".into(),
                audience: "my-api".into(),
                jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            })
            .await;

        let config = result.unwrap();
        assert_eq!(config.issuer_url(), "https://auth.example.com");
        assert_eq!(config.audience(), "my-api");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "oidc_config_store:insert",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn rejects_empty_issuer_url() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOidcConfigHandler::new(factory);

        let result = handler
            .handle(CreateOidcConfig {
                org_id: OrganizationId::new(),
                issuer_url: "".into(),
                audience: "my-api".into(),
                jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }

    #[tokio::test]
    async fn rejects_empty_audience() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOidcConfigHandler::new(factory);

        let result = handler
            .handle(CreateOidcConfig {
                org_id: OrganizationId::new(),
                issuer_url: "https://auth.example.com".into(),
                audience: "".into(),
                jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }

    #[tokio::test]
    async fn rejects_empty_jwks_url() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOidcConfigHandler::new(factory);

        let result = handler
            .handle(CreateOidcConfig {
                org_id: OrganizationId::new(),
                issuer_url: "https://auth.example.com".into(),
                audience: "my-api".into(),
                jwks_url: "".into(),
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }
}
