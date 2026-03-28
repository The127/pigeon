use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::EventTypeReadStore;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::{EventType, EventTypeId, EventTypeState};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

pub struct PgEventTypeReadStore {
    pool: PgPool,
}

impl PgEventTypeReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventTypeReadStore for PgEventTypeReadStore {
    async fn find_by_id(
        &self,
        id: &EventTypeId,
        org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError> {
        let row = sqlx::query_as::<_, EventTypeRow>(
            "SELECT et.id, et.app_id, et.name, et.schema, et.system, et.created_at, et.xmin::text::bigint AS version \
             FROM event_types et \
             JOIN applications a ON a.id = et.app_id \
             WHERE et.id = $1 AND a.org_id = $2 AND et.system = false",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_event_type()))
    }

    async fn find_by_app_and_name(
        &self,
        app_id: &ApplicationId,
        name: &str,
        org_id: &OrganizationId,
    ) -> Result<Option<EventType>, ApplicationError> {
        let row = sqlx::query_as::<_, EventTypeRow>(
            "SELECT et.id, et.app_id, et.name, et.schema, et.system, et.created_at, et.xmin::text::bigint AS version \
             FROM event_types et \
             JOIN applications a ON a.id = et.app_id \
             WHERE et.app_id = $1 AND et.name = $2 AND a.org_id = $3",
        )
        .bind(app_id.as_uuid())
        .bind(name)
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_event_type()))
    }

    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<EventType>, ApplicationError> {
        let rows = sqlx::query_as::<_, EventTypeRow>(
            "SELECT et.id, et.app_id, et.name, et.schema, et.system, et.created_at, et.xmin::text::bigint AS version \
             FROM event_types et \
             JOIN applications a ON a.id = et.app_id \
             WHERE et.app_id = $1 AND a.org_id = $2 AND et.system = false \
             ORDER BY et.created_at DESC \
             LIMIT $3 OFFSET $4",
        )
        .bind(app_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_event_type()).collect())
    }

    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        let row: (i64,) =
            sqlx::query_as(
                "SELECT COUNT(*) FROM event_types et \
                 JOIN applications a ON a.id = et.app_id \
                 WHERE et.app_id = $1 AND a.org_id = $2 AND et.system = false",
            )
                .bind(app_id.as_uuid())
                .bind(org_id.as_uuid())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }
}

#[derive(sqlx::FromRow)]
struct EventTypeRow {
    id: uuid::Uuid,
    app_id: uuid::Uuid,
    name: String,
    schema: Option<serde_json::Value>,
    system: bool,
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
            system: self.system,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}
