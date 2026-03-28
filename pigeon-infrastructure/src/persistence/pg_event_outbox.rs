use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::event_dispatcher::{EventOutbox, OutboxEntry};
use pigeon_domain::event::DomainEvent;
use pigeon_domain::outbox::OutboxEntryId;

pub struct PgEventOutbox {
    pool: PgPool,
}

impl PgEventOutbox {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: uuid::Uuid,
    event_type: String,
    payload: serde_json::Value,
}

#[async_trait]
impl EventOutbox for PgEventOutbox {
    async fn poll(&self, limit: u32) -> Result<Vec<OutboxEntry>, ApplicationError> {
        let rows = sqlx::query_as::<_, OutboxRow>(
            "SELECT id, event_type, payload \
             FROM event_outbox \
             WHERE processed_at IS NULL \
             ORDER BY created_at \
             LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            if let Some(event) = DomainEvent::from_outbox(&row.event_type, &row.payload) {
                entries.push(OutboxEntry {
                    id: OutboxEntryId::from_uuid(row.id),
                    event,
                });
            }
        }

        Ok(entries)
    }

    async fn mark_processed(&self, id: &OutboxEntryId) -> Result<(), ApplicationError> {
        sqlx::query("UPDATE event_outbox SET processed_at = now() WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }
}
