use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::OrganizationStore;
use pigeon_domain::organization::{Organization, OrganizationId, OrganizationState};
use pigeon_domain::version::Version;
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgOrganizationStore {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgOrganizationStore {
    pub(crate) fn new(pool: PgPool, tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { pool, tracker }
    }
}

#[async_trait]
impl OrganizationStore for PgOrganizationStore {
    async fn insert(&mut self, organization: &Organization) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertOrganization(organization.clone()));
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &OrganizationId,
    ) -> Result<Option<Organization>, ApplicationError> {
        // Check change tracker first
        {
            let tracker = self.tracker.lock().unwrap();
            if let Some(pending) = tracker.find_pending_organization(id) {
                return Ok(pending.cloned());
            }
        }

        // Fall back to DB
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

    async fn save(&mut self, organization: &Organization) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::SaveOrganization(organization.clone()));
        Ok(())
    }

    async fn delete(&mut self, id: &OrganizationId) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::DeleteOrganization(id.clone()));
        Ok(())
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
