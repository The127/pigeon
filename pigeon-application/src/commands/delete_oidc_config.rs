use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::oidc_config::OidcConfigId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct DeleteOidcConfig {
    pub id: OidcConfigId,
}

impl Command for DeleteOidcConfig {
    type Output = ();

    fn command_name(&self) -> &'static str {
        "DeleteOidcConfig"
    }
}

pub struct DeleteOidcConfigHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl DeleteOidcConfigHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<DeleteOidcConfig> for DeleteOidcConfigHandler {
    async fn handle(&self, command: DeleteOidcConfig) -> Result<(), ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let existing = uow
            .oidc_config_store()
            .find_by_id(&command.id)
            .await?;

        if existing.is_none() {
            return Err(ApplicationError::NotFound);
        }

        uow.oidc_config_store().delete(&command.id).await?;
        uow.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::oidc_config::OidcConfig;
    use pigeon_domain::organization::OrganizationId;

    #[tokio::test]
    async fn deletes_oidc_config_successfully() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let config = OidcConfig::new(
            OrganizationId::new(),
            "https://auth.example.com".into(),
            "my-api".into(),
            "https://auth.example.com/.well-known/jwks.json".into(),
        )
        .unwrap();
        let id = config.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.oidc_config_store().insert(&config).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = DeleteOidcConfigHandler::new(factory);
        let result = handler.handle(DeleteOidcConfig { id }).await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "oidc_config_store:find_by_id",
                "oidc_config_store:delete",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_config_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = DeleteOidcConfigHandler::new(factory);
        let result = handler
            .handle(DeleteOidcConfig {
                id: OidcConfigId::new(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }
}
