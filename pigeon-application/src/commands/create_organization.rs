use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::oidc_config::OidcConfig;
use pigeon_domain::organization::Organization;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::stores::OrganizationReadStore;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct CreateOrganization {
    pub name: String,
    pub slug: String,
    pub oidc_issuer_url: String,
    pub oidc_audience: String,
    pub oidc_jwks_url: String,
}

impl Command for CreateOrganization {
    type Output = Organization;

    fn command_name(&self) -> &'static str {
        "CreateOrganization"
    }
}

pub struct CreateOrganizationHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
    org_read_store: Arc<dyn OrganizationReadStore>,
}

impl CreateOrganizationHandler {
    pub fn new(
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        org_read_store: Arc<dyn OrganizationReadStore>,
    ) -> Self {
        Self { uow_factory, org_read_store }
    }
}

#[async_trait]
impl CommandHandler<CreateOrganization> for CreateOrganizationHandler {
    async fn handle(&self, command: CreateOrganization) -> Result<Organization, ApplicationError> {
        let org = Organization::new(command.name, command.slug)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let oidc_config = OidcConfig::new(
            org.id().clone(),
            command.oidc_issuer_url,
            command.oidc_audience,
            command.oidc_jwks_url,
        )
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        if self.org_read_store.find_by_slug(org.slug()).await?.is_some() {
            return Err(ApplicationError::Validation(
                "Organization with this slug already exists".to_string(),
            ));
        }

        let mut uow = self.uow_factory.begin().await?;
        uow.organization_store().insert(&org).await?;
        uow.oidc_config_store().insert(&oidc_config).await?;
        uow.commit().await?;

        Ok(org)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{
        FakeOrganizationReadStore, FakeUnitOfWorkFactory, OperationLog, SharedOrganizationData,
    };

    fn empty_org_store(log: &OperationLog) -> Arc<dyn OrganizationReadStore> {
        Arc::new(FakeOrganizationReadStore::new(log.clone(), SharedOrganizationData::default()))
    }

    #[tokio::test]
    async fn creates_organization_with_oidc_config() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(factory.clone(), empty_org_store(&log));

        let result = handler
            .handle(CreateOrganization {
                name: "my-org".into(),
                slug: "my-org".into(),
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            })
            .await;

        let org = result.unwrap();
        assert_eq!(org.name(), "my-org");
        assert_eq!(org.slug(), "my-org");
        assert_eq!(
            log.entries(),
            vec![
                "organization_read_store:find_by_slug",
                "uow_factory:begin",
                "organization_store:insert",
                "oidc_config_store:insert",
                "uow:commit",
            ]
        );

        // Verify OIDC config was persisted
        let configs = factory.oidc_config_data().oidc_configs.lock().unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].org_id(), org.id());
        assert_eq!(configs[0].issuer_url(), "https://auth.example.com");
        assert_eq!(configs[0].audience(), "pigeon-api");
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(factory, empty_org_store(&log));

        let result = handler
            .handle(CreateOrganization {
                name: "".into(),
                slug: "my-org".into(),
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }

    #[tokio::test]
    async fn rejects_invalid_slug() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(factory, empty_org_store(&log));

        let result = handler
            .handle(CreateOrganization {
                name: "My Org".into(),
                slug: "My Org".into(),
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            })
            .await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
    }

    #[tokio::test]
    async fn rejects_empty_oidc_issuer() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(factory, empty_org_store(&log));

        let result = handler
            .handle(CreateOrganization {
                name: "my-org".into(),
                slug: "my-org".into(),
                oidc_issuer_url: "".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
        assert!(log.entries().is_empty());
    }

    #[tokio::test]
    async fn rejects_duplicate_slug() {
        let log = OperationLog::new();
        let org_data = SharedOrganizationData::default();
        let existing = pigeon_domain::organization::Organization::new("existing".into(), "taken-slug".into()).unwrap();
        org_data.organizations.lock().unwrap().push(existing);
        let org_store = Arc::new(FakeOrganizationReadStore::new(log.clone(), org_data));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(factory, org_store);

        let result = handler
            .handle(CreateOrganization {
                name: "new-org".into(),
                slug: "taken-slug".into(),
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }
}
