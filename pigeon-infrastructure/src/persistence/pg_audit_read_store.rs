use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::audit_read_store::{AuditLogEntry, AuditReadStore};
use pigeon_domain::organization::OrganizationId;

pub struct PgAuditReadStore {
    pool: PgPool,
}

impl PgAuditReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: uuid::Uuid,
    command_name: String,
    actor: String,
    org_id: uuid::Uuid,
    timestamp: DateTime<Utc>,
    success: bool,
    error_message: Option<String>,
}

#[async_trait]
impl AuditReadStore for PgAuditReadStore {
    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<AuditLogEntry>, ApplicationError> {
        let rows = sqlx::query_as::<_, AuditLogRow>(
            "SELECT id, command_name, actor, org_id, timestamp, success, error_message \
             FROM audit_log \
             WHERE org_id = $1 \
             ORDER BY timestamp DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(org_id.as_uuid())
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| AuditLogEntry {
                id: r.id,
                command_name: r.command_name,
                actor: r.actor,
                org_id: OrganizationId::from_uuid(r.org_id),
                timestamp: r.timestamp,
                success: r.success,
                error_message: r.error_message,
            })
            .collect())
    }

    async fn count_by_org(
        &self,
        org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE org_id = $1",
        )
        .bind(org_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }
}
