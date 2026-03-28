use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::EventTypeStore;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::{EventType, EventTypeId, EventTypeState};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgEventTypeStore {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgEventTypeStore {
    pub(crate) fn new(pool: PgPool, tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { pool, tracker }
    }
}

#[async_trait]
impl EventTypeStore for PgEventTypeStore {
    async fn insert(&mut self, event_type: &EventType) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertEventType(event_type.clone()));
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &EventTypeId,
        org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError> {
        // Check change tracker first
        {
            let tracker = self.tracker.lock().unwrap();
            if let Some(pending) = tracker.find_pending_event_type(id) {
                return Ok(pending.cloned());
            }
        }

        // Fall back to DB with org scoping via JOIN
        let row = sqlx::query_as::<_, EventTypeRow>(
            "SELECT et.id, et.app_id, et.name, et.schema, et.created_at, et.xmin::text::bigint AS version \
             FROM event_types et \
             JOIN applications a ON a.id = et.app_id \
             WHERE et.id = $1 AND a.org_id = $2",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_event_type()))
    }

    async fn save(&mut self, event_type: &EventType) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::SaveEventType(event_type.clone()));
        Ok(())
    }

    async fn delete(&mut self, id: &EventTypeId) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::DeleteEventType(id.clone()));
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct EventTypeRow {
    id: uuid::Uuid,
    app_id: uuid::Uuid,
    name: String,
    schema: Option<serde_json::Value>,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl EventTypeRow {
    fn into_event_type(self) -> EventType {
        EventType::reconstitute(EventTypeState {
            id: EventTypeId::from_uuid(self.id),
            app_id: ApplicationId::from_uuid(self.app_id),
            name: self.name,
            schema: self.schema,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}
