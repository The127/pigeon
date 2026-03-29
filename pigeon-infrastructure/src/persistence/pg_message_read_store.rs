use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::message_status::MessageWithStatus;
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
struct MessageWithStatusRow {
    id: uuid::Uuid,
    app_id: uuid::Uuid,
    event_type_id: uuid::Uuid,
    payload: serde_json::Value,
    idempotency_key: String,
    idempotency_expires_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
    // From message_delivery_status (nullable via LEFT JOIN)
    attempts_created: Option<i32>,
    succeeded: Option<i32>,
    failed: Option<i32>,
    dead_lettered: Option<i32>,
}

impl MessageWithStatusRow {
    fn into_message_with_status(self) -> MessageWithStatus {
        let message = Message::reconstitute(MessageState {
            id: MessageId::from_uuid(self.id),
            app_id: ApplicationId::from_uuid(self.app_id),
            event_type_id: EventTypeId::from_uuid(self.event_type_id),
            payload: self.payload,
            idempotency_key: IdempotencyKey::new(self.idempotency_key),
            idempotency_expires_at: self.idempotency_expires_at,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        });
        MessageWithStatus {
            message,
            attempts_created: self.attempts_created.unwrap_or(0) as u32,
            succeeded: self.succeeded.unwrap_or(0) as u32,
            failed: self.failed.unwrap_or(0) as u32,
            dead_lettered: self.dead_lettered.unwrap_or(0) as u32,
        }
    }
}

const SELECT_WITH_STATUS: &str = "\
    SELECT m.id, m.app_id, m.event_type_id, m.payload, \
           m.idempotency_key, m.idempotency_expires_at, m.created_at, \
           m.xmin::text::bigint AS version, \
           mds.attempts_created, mds.succeeded, mds.failed, mds.dead_lettered \
    FROM messages m \
    JOIN applications a ON a.id = m.app_id \
    LEFT JOIN message_delivery_status mds ON mds.message_id = m.id";

#[async_trait]
impl MessageReadStore for PgMessageReadStore {
    async fn find_by_id(
        &self,
        id: &MessageId,
        org_id: &OrganizationId,
    ) -> Result<Option<MessageWithStatus>, ApplicationError> {
        let query = format!("{SELECT_WITH_STATUS} WHERE m.id = $1 AND a.org_id = $2");
        let row = sqlx::query_as::<_, MessageWithStatusRow>(&query)
            .bind(id.as_uuid())
            .bind(org_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_message_with_status()))
    }

    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<MessageWithStatus>, ApplicationError> {
        let query = format!(
            "{SELECT_WITH_STATUS} WHERE m.app_id = $1 AND a.org_id = $2 \
             ORDER BY m.created_at DESC LIMIT $3 OFFSET $4"
        );
        let rows = sqlx::query_as::<_, MessageWithStatusRow>(&query)
            .bind(app_id.as_uuid())
            .bind(org_id.as_uuid())
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_message_with_status()).collect())
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
