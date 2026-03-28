use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::{InsertMessageResult, MessageStore};
use pigeon_domain::message::{IdempotencyKey, Message, MessageId, MessageState};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgMessageStore {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgMessageStore {
    pub(crate) fn new(pool: PgPool, tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { pool, tracker }
    }
}

#[async_trait]
impl MessageStore for PgMessageStore {
    async fn insert_or_get_existing(
        &mut self,
        message: &Message,
        org_id: &OrganizationId,
    ) -> Result<InsertMessageResult, ApplicationError> {
        // Check DB for existing message by (app_id, idempotency_key) where not expired
        // JOIN through applications to verify org ownership
        let row = sqlx::query_as::<_, MessageRow>(
            "SELECT m.id, m.app_id, m.event_type_id, m.payload, m.idempotency_key, \
             m.idempotency_expires_at, m.created_at \
             FROM messages m \
             JOIN applications a ON a.id = m.app_id \
             WHERE m.app_id = $1 AND m.idempotency_key = $2 AND m.idempotency_expires_at > now() AND a.org_id = $3",
        )
        .bind(message.app_id().as_uuid())
        .bind(message.idempotency_key().as_str())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        if let Some(row) = row {
            return Ok(InsertMessageResult {
                message: row.into_message(),
                was_existing: true,
            });
        }

        // Record for flush
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertMessage(message.clone()));

        Ok(InsertMessageResult {
            message: message.clone(),
            was_existing: false,
        })
    }

    async fn expire_idempotency_keys(
        &self,
        now: DateTime<Utc>,
    ) -> Result<u64, ApplicationError> {
        let result = sqlx::query(
            "DELETE FROM messages WHERE idempotency_expires_at <= $1",
        )
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: uuid::Uuid,
    app_id: uuid::Uuid,
    event_type_id: uuid::Uuid,
    payload: serde_json::Value,
    idempotency_key: String,
    idempotency_expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
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
            version: Version::new(0),
        })
    }
}
