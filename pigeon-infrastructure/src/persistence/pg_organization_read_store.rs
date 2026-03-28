use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::OrganizationReadStore;
use pigeon_domain::organization::{Organization, OrganizationId, OrganizationState};
use pigeon_domain::version::Version;
use sqlx::PgPool;

pub struct PgOrganizationReadStore {
    pool: PgPool,
}

impl PgOrganizationReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationReadStore for PgOrganizationReadStore {
    async fn find_by_id(
        &self,
        id: &OrganizationId,
    ) -> Result<Option<Organization>, ApplicationError> {
        let row = sqlx::query_as::<_, OrganizationRow>(
            "SELECT id, name, slug, created_at, xmin::text::bigint AS version \
             FROM organizations WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_organization()))
    }

    async fn list(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Organization>, ApplicationError> {
        let rows = sqlx::query_as::<_, OrganizationRow>(
            "SELECT id, name, slug, created_at, xmin::text::bigint AS version \
             FROM organizations \
             ORDER BY created_at DESC \
             LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_organization()).collect())
    }

    async fn count(&self) -> Result<u64, ApplicationError> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM organizations")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }
}

#[derive(sqlx::FromRow)]
struct OrganizationRow {
    id: uuid::Uuid,
    name: String,
    slug: String,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl OrganizationRow {
    fn into_organization(self) -> Organization {
        Organization::reconstitute(OrganizationState {
            id: OrganizationId::from_uuid(self.id),
            name: self.name,
            slug: self.slug,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}
