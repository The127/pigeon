use pigeon_application::ports::stores::OrganizationReadStore;
use pigeon_application::ports::unit_of_work::UnitOfWorkFactory;
use pigeon_domain::oidc_config::OidcConfig;
use pigeon_domain::organization::Organization;
use tracing::info;

use crate::config::PigeonConfig;

pub(crate) async fn bootstrap_organization(
    uow_factory: &dyn UnitOfWorkFactory,
    org_read_store: &dyn OrganizationReadStore,
    config: &PigeonConfig,
) -> anyhow::Result<()> {
    if !config.bootstrap_org_enabled {
        return Ok(());
    }

    let count = org_read_store
        .count()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count organizations: {e}"))?;

    if count > 0 {
        info!("Organizations already exist, skipping bootstrap");
        return Ok(());
    }

    let org = Organization::new(
        config.bootstrap_org_name.clone(),
        config.bootstrap_org_slug.clone(),
    )
    .map_err(|e| anyhow::anyhow!("Invalid bootstrap org config: {e}"))?;

    let jwks_url = if config.bootstrap_oidc_jwks_url.is_empty() {
        format!(
            "{}/.well-known/jwks.json",
            config.bootstrap_oidc_issuer_url.trim_end_matches('/')
        )
    } else {
        config.bootstrap_oidc_jwks_url.clone()
    };

    let oidc_config = OidcConfig::new(
        org.id().clone(),
        config.bootstrap_oidc_issuer_url.clone(),
        config.bootstrap_oidc_audience.clone(),
        jwks_url,
    )
    .map_err(|e| anyhow::anyhow!("Invalid bootstrap OIDC config: {e}"))?;

    let mut uow = uow_factory
        .begin()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to begin UoW: {e}"))?;
    uow.organization_store()
        .insert(&org)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert org: {e}"))?;
    uow.oidc_config_store()
        .insert(&oidc_config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert OIDC config: {e}"))?;
    uow.commit()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to commit: {e}"))?;

    info!(
        org_id = %org.id().as_uuid(),
        slug = %org.slug(),
        issuer_url = %oidc_config.issuer_url(),
        audience = %oidc_config.audience(),
        "Bootstrap organization created with OIDC config"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use pigeon_application::test_support::fakes::{
        FakeOrganizationReadStore, FakeUnitOfWorkFactory, OperationLog, SharedOrganizationData,
    };
    use pigeon_domain::organization::Organization;

    use super::*;

    fn config_enabled() -> PigeonConfig {
        PigeonConfig {
            database_url: "postgres://localhost/test".to_string(),
            listen_addr: "0.0.0.0:3000".to_string(),
            bootstrap_org_enabled: true,
            bootstrap_org_name: "System".to_string(),
            bootstrap_org_slug: "system".to_string(),
            bootstrap_oidc_issuer_url: "https://auth.example.com".to_string(),
            bootstrap_oidc_audience: "pigeon-api".to_string(),
            bootstrap_oidc_jwks_url: String::new(),
            jwks_cache_ttl: Duration::from_secs(3600),
            worker_batch_size: 10,
            worker_poll_interval: Duration::from_millis(1000),
            worker_max_retries: 5,
            worker_backoff_base_secs: 30,
            worker_max_backoff_secs: 3600,
            worker_http_timeout: Duration::from_secs(30),
            worker_cleanup_interval_secs: 3600,
            worker_auto_disable_threshold: 5,
        }
    }

    fn config_disabled() -> PigeonConfig {
        PigeonConfig {
            bootstrap_org_enabled: false,
            ..config_enabled()
        }
    }

    #[tokio::test]
    async fn creates_org_and_oidc_config_when_none_exist() {
        let log = OperationLog::new();
        let org_data = SharedOrganizationData::default();
        let factory = Arc::new(FakeUnitOfWorkFactory::with_organization_data(
            log.clone(),
            org_data.clone(),
        ));
        let read_store = FakeOrganizationReadStore::new(log.clone(), org_data.clone());
        let config = config_enabled();

        bootstrap_organization(factory.as_ref(), &read_store, &config)
            .await
            .unwrap();

        let orgs = org_data.organizations.lock().unwrap();
        assert_eq!(orgs.len(), 1);
        assert_eq!(orgs[0].name(), "System");
        assert_eq!(orgs[0].slug(), "system");

        let oidc_configs = factory.oidc_config_data().oidc_configs.lock().unwrap();
        assert_eq!(oidc_configs.len(), 1);
        assert_eq!(oidc_configs[0].issuer_url(), "https://auth.example.com");
        assert_eq!(oidc_configs[0].audience(), "pigeon-api");
        assert_eq!(
            oidc_configs[0].jwks_url(),
            "https://auth.example.com/.well-known/jwks.json"
        );
    }

    #[tokio::test]
    async fn uses_explicit_jwks_url_when_provided() {
        let log = OperationLog::new();
        let org_data = SharedOrganizationData::default();
        let factory = Arc::new(FakeUnitOfWorkFactory::with_organization_data(
            log.clone(),
            org_data.clone(),
        ));
        let read_store = FakeOrganizationReadStore::new(log.clone(), org_data.clone());
        let mut config = config_enabled();
        config.bootstrap_oidc_jwks_url = "https://custom.example.com/jwks".to_string();

        bootstrap_organization(factory.as_ref(), &read_store, &config)
            .await
            .unwrap();

        let oidc_configs = factory.oidc_config_data().oidc_configs.lock().unwrap();
        assert_eq!(oidc_configs[0].jwks_url(), "https://custom.example.com/jwks");
    }

    #[tokio::test]
    async fn skips_when_orgs_already_exist() {
        let log = OperationLog::new();
        let org_data = SharedOrganizationData::default();

        let existing = Organization::new("Existing".to_string(), "existing".to_string()).unwrap();
        org_data.organizations.lock().unwrap().push(existing);

        let factory = Arc::new(FakeUnitOfWorkFactory::with_organization_data(
            log.clone(),
            org_data.clone(),
        ));
        let read_store = FakeOrganizationReadStore::new(log.clone(), org_data.clone());
        let config = config_enabled();

        bootstrap_organization(factory.as_ref(), &read_store, &config)
            .await
            .unwrap();

        let orgs = org_data.organizations.lock().unwrap();
        assert_eq!(orgs.len(), 1);
        assert_eq!(orgs[0].slug(), "existing");
    }

    #[tokio::test]
    async fn respects_enabled_false() {
        let log = OperationLog::new();
        let org_data = SharedOrganizationData::default();
        let factory = Arc::new(FakeUnitOfWorkFactory::with_organization_data(
            log.clone(),
            org_data.clone(),
        ));
        let read_store = FakeOrganizationReadStore::new(log.clone(), org_data.clone());
        let config = config_disabled();

        bootstrap_organization(factory.as_ref(), &read_store, &config)
            .await
            .unwrap();

        let orgs = org_data.organizations.lock().unwrap();
        assert_eq!(orgs.len(), 0);
        assert!(log.entries().is_empty());
    }
}
