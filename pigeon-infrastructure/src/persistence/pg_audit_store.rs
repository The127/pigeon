use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::audit_store::{AuditEntry, AuditStore};

pub struct PgAuditStore {
    pool: PgPool,
}

impl PgAuditStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditStore for PgAuditStore {
    async fn record(&self, entry: AuditEntry) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO audit_log (command_name, actor, org_id, timestamp, success, error_message) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(entry.command_name)
        .bind(&entry.actor)
        .bind(entry.org_id.as_uuid())
        .bind(entry.timestamp)
        .bind(entry.success)
        .bind(&entry.error_message)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }
}
