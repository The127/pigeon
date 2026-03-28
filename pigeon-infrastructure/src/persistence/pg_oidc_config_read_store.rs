use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::OidcConfigReadStore;
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId, OidcConfigState};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

pub struct PgOidcConfigReadStore {
    pool: PgPool,
}

impl PgOidcConfigReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OidcConfigReadStore for PgOidcConfigReadStore {
    async fn find_by_issuer_and_audience(
        &self,
        issuer_url: &str,
        audience: &str,
    ) -> Result<Option<OidcConfig>, ApplicationError> {
        let row = sqlx::query_as::<_, OidcConfigRow>(
            "SELECT id, org_id, issuer_url, audience, jwks_url, created_at, \
             xmin::text::bigint AS version \
             FROM oidc_configs WHERE issuer_url = $1 AND audience = $2",
        )
        .bind(issuer_url)
        .bind(audience)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_oidc_config()))
    }

    async fn find_by_id(
        &self,
        id: &OidcConfigId,
    ) -> Result<Option<OidcConfig>, ApplicationError> {
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

    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<OidcConfig>, ApplicationError> {
        let rows = sqlx::query_as::<_, OidcConfigRow>(
            "SELECT id, org_id, issuer_url, audience, jwks_url, created_at, \
             xmin::text::bigint AS version \
             FROM oidc_configs \
             WHERE org_id = $1 \
             ORDER BY created_at DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(org_id.as_uuid())
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_oidc_config()).collect())
    }

    async fn count_by_org(
        &self,
        org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM oidc_configs WHERE org_id = $1")
                .bind(org_id.as_uuid())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
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
