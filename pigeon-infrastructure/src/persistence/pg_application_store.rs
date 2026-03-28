use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::ApplicationStore;
use pigeon_domain::application::{Application, ApplicationId, ApplicationState};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgApplicationStore {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgApplicationStore {
    pub(crate) fn new(pool: PgPool, tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { pool, tracker }
    }
}

#[async_trait]
impl ApplicationStore for PgApplicationStore {
    async fn insert(&mut self, application: &Application) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertApplication(application.clone()));
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &ApplicationId,
    ) -> Result<Option<Application>, ApplicationError> {
        // Check change tracker first
        {
            let tracker = self.tracker.lock().unwrap();
            if let Some(pending) = tracker.find_pending_application(id) {
                return Ok(pending.cloned());
            }
        }

        // Fall back to DB (outside any transaction — direct pool query)
        let row = sqlx::query_as::<_, ApplicationRow>(
            "SELECT id, org_id, name, uid, created_at, xmin::text::bigint AS version \
             FROM applications WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_application()))
    }

    async fn save(&mut self, application: &Application) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::SaveApplication(application.clone()));
        Ok(())
    }

    async fn delete(&mut self, id: &ApplicationId) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::DeleteApplication(id.clone()));
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ApplicationRow {
    id: uuid::Uuid,
    org_id: uuid::Uuid,
    name: String,
    uid: String,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl ApplicationRow {
    fn into_application(self) -> Application {
        Application::reconstitute(ApplicationState {
            id: ApplicationId::from_uuid(self.id),
            org_id: OrganizationId::from_uuid(self.org_id),
            name: self.name,
            uid: self.uid,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}
