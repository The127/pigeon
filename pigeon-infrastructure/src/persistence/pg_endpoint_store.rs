use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::EndpointStore;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::{Endpoint, EndpointId, EndpointState};
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgEndpointStore {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgEndpointStore {
    pub(crate) fn new(pool: PgPool, tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { pool, tracker }
    }
}

#[async_trait]
impl EndpointStore for PgEndpointStore {
    async fn insert(&mut self, endpoint: &Endpoint) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertEndpoint(endpoint.clone()));
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &EndpointId,
        org_id: &OrganizationId,
    ) -> Result<Option<Endpoint>, ApplicationError> {
        // Check change tracker first
        {
            let tracker = self.tracker.lock().unwrap();
            if let Some(pending) = tracker.find_pending_endpoint(id) {
                return Ok(pending.cloned());
            }
        }

        // Fall back to DB with org scoping via JOIN
        let row = sqlx::query_as::<_, EndpointRow>(
            "SELECT e.id, e.app_id, e.url, e.signing_secret, e.enabled, e.created_at, \
             e.xmin::text::bigint AS version \
             FROM endpoints e \
             JOIN applications a ON a.id = e.app_id \
             WHERE e.id = $1 AND a.org_id = $2",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        match row {
            Some(row) => {
                let event_type_ids = sqlx::query_scalar::<_, uuid::Uuid>(
                    "SELECT event_type_id FROM endpoint_events WHERE endpoint_id = $1",
                )
                .bind(row.id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| ApplicationError::Internal(e.to_string()))?;

                Ok(Some(row.into_endpoint(
                    event_type_ids
                        .into_iter()
                        .map(EventTypeId::from_uuid)
                        .collect(),
                )))
            }
            None => Ok(None),
        }
    }

    async fn save(&mut self, endpoint: &Endpoint) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::SaveEndpoint(endpoint.clone()));
        Ok(())
    }

    async fn delete(&mut self, id: &EndpointId) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::DeleteEndpoint(id.clone()));
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct EndpointRow {
    id: uuid::Uuid,
    app_id: uuid::Uuid,
    url: String,
    signing_secret: String,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl EndpointRow {
    fn into_endpoint(self, event_type_ids: Vec<EventTypeId>) -> Endpoint {
        Endpoint::reconstitute(EndpointState {
            id: EndpointId::from_uuid(self.id),
            app_id: ApplicationId::from_uuid(self.app_id),
            url: self.url,
            signing_secret: self.signing_secret,
            enabled: self.enabled,
            event_type_ids,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}
