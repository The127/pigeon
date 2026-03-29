use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::OidcConfigStore;
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId, OidcConfigState};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgOidcConfigStore {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgOidcConfigStore {
    pub(crate) fn new(pool: PgPool, tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { pool, tracker }
    }
}

#[async_trait]
impl OidcConfigStore for PgOidcConfigStore {
    async fn insert(&mut self, config: &OidcConfig) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertOidcConfig(config.clone()));
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &OidcConfigId,
    ) -> Result<Option<OidcConfig>, ApplicationError> {
        // Check change tracker first
        {
            let tracker = self.tracker.lock().unwrap();
            if let Some(pending) = tracker.find_pending_oidc_config(id) {
                return Ok(pending.cloned());
            }
        }

        // Fall back to DB
        let row = sqlx::query_as::<_, OidcConfigRow>(
            "SELECT id, org_id, issuer_url, audience, jwks_url, created_at, \
             xmin::text::bigint AS version \
             FROM oidc_configs WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_oidc_config()))
    }

    async fn count_by_org(&self, org_id: &OrganizationId) -> Result<u64, ApplicationError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM oidc_configs WHERE org_id = $1",
        )
        .bind(org_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(count as u64)
    }

    async fn delete(&mut self, id: &OidcConfigId) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::DeleteOidcConfig(id.clone()));
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct OidcConfigRow {
    id: uuid::Uuid,
    org_id: uuid::Uuid,
    issuer_url: String,
    audience: String,
    jwks_url: String,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl OidcConfigRow {
    fn into_oidc_config(self) -> OidcConfig {
        OidcConfig::reconstitute(OidcConfigState {
            id: OidcConfigId::from_uuid(self.id),
            org_id: OrganizationId::from_uuid(self.org_id),
            issuer_url: self.issuer_url,
            audience: self.audience,
            jwks_url: self.jwks_url,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}
