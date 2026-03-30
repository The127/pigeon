use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::oidc_config::OidcConfig;
use pigeon_domain::organization::Organization;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;
use crate::ports::stores::OrganizationReadStore;

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
    org_read_store: Arc<dyn OrganizationReadStore>,
}

impl CreateOrganizationHandler {
    pub fn new(
        org_read_store: Arc<dyn OrganizationReadStore>,
    ) -> Self {
        Self { org_read_store }
    }
}

#[async_trait]
impl CommandHandler<CreateOrganization> for CreateOrganizationHandler {
    async fn handle(&self, command: CreateOrganization, ctx: &mut RequestContext) -> Result<Organization, ApplicationError> {
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

        ctx.uow().organization_store().insert(&org).await?;
        ctx.uow().oidc_config_store().insert(&oidc_config).await?;

        Ok(org)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mediator::pipeline::RequestContext;
    use crate::ports::unit_of_work::UnitOfWorkFactory;
    use crate::test_support::fakes::{
        FakeOrganizationReadStore, FakeUnitOfWorkFactory, OperationLog, SharedOrganizationData,
    };
    use pigeon_domain::organization::OrganizationId;

    fn empty_org_store(log: &OperationLog) -> Arc<dyn OrganizationReadStore> {
        Arc::new(FakeOrganizationReadStore::new(log.clone(), SharedOrganizationData::default()))
    }

    #[tokio::test]
    async fn creates_organization_with_oidc_config() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(empty_org_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOrganization {
                name: "my-org".into(),
                slug: "my-org".into(),
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await;

        let org = result.unwrap();
        assert_eq!(org.name(), "my-org");
        assert_eq!(org.slug(), "my-org");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "organization_read_store:find_by_slug",
                "organization_store:insert",
                "oidc_config_store:insert",
            ]
        );
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(empty_org_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOrganization {
                name: "".into(),
                slug: "my-org".into(),
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_invalid_slug() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(empty_org_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOrganization {
                name: "My Org".into(),
                slug: "My Org".into(),
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_empty_oidc_issuer() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(empty_org_store(&log));

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOrganization {
                name: "my-org".into(),
                slug: "my-org".into(),
                oidc_issuer_url: "".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }

    #[tokio::test]
    async fn rejects_duplicate_slug() {
        let log = OperationLog::new();
        let org_data = SharedOrganizationData::default();
        let existing = pigeon_domain::organization::Organization::new("existing".into(), "taken-slug".into()).unwrap();
        org_data.organizations.lock().unwrap().push(existing);
        let org_store = Arc::new(FakeOrganizationReadStore::new(log.clone(), org_data));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateOrganizationHandler::new(org_store);

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(CreateOrganization {
                name: "new-org".into(),
                slug: "taken-slug".into(),
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }
}
