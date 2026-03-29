use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::MessageReadStore;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::message::{IdempotencyKey, Message, MessageId, MessageState};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;

pub struct PgMessageReadStore {
    pool: PgPool,
}

impl PgMessageReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: uuid::Uuid,
    app_id: uuid::Uuid,
    event_type_id: uuid::Uuid,
    payload: serde_json::Value,
    idempotency_key: String,
    idempotency_expires_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl MessageRow {
    fn into_message(self) -> Message {
        Message::reconstitute(MessageState {
            id: MessageId::from_uuid(self.id),
            app_id: ApplicationId::from_uuid(self.app_id),
            event_type_id: EventTypeId::from_uuid(self.event_type_id),
            payload: self.payload,
            idempotency_key: IdempotencyKey::new(self.idempotency_key),
            idempotency_expires_at: self.idempotency_expires_at,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}

#[async_trait]
impl MessageReadStore for PgMessageReadStore {
    async fn find_by_id(
        &self,
        id: &MessageId,
        org_id: &OrganizationId,
    ) -> Result<Option<Message>, ApplicationError> {
        let row = sqlx::query_as::<_, MessageRow>(
            "SELECT m.id, m.app_id, m.event_type_id, m.payload, \
                    m.idempotency_key, m.idempotency_expires_at, m.created_at, \
                    m.xmin::text::bigint AS version \
             FROM messages m \
             JOIN applications a ON a.id = m.app_id \
             WHERE m.id = $1 AND a.org_id = $2",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_message()))
    }

    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Message>, ApplicationError> {
        let rows = sqlx::query_as::<_, MessageRow>(
            "SELECT m.id, m.app_id, m.event_type_id, m.payload, \
                    m.idempotency_key, m.idempotency_expires_at, m.created_at, \
                    m.xmin::text::bigint AS version \
             FROM messages m \
             JOIN applications a ON a.id = m.app_id \
             WHERE m.app_id = $1 AND a.org_id = $2 \
             ORDER BY m.created_at DESC \
             LIMIT $3 OFFSET $4",
        )
        .bind(app_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_message()).collect())
    }

    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM messages m \
             JOIN applications a ON a.id = m.app_id \
             WHERE m.app_id = $1 AND a.org_id = $2",
        )
        .bind(app_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }
}
