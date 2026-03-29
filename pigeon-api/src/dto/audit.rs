use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::ports::audit_read_store::AuditLogEntry;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditLogResponse {
    pub id: Uuid,
    pub command_name: String,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl From<AuditLogEntry> for AuditLogResponse {
    fn from(entry: AuditLogEntry) -> Self {
        Self {
            id: entry.id,
            command_name: entry.command_name,
            actor: entry.actor,
            timestamp: entry.timestamp,
            success: entry.success,
            error_message: entry.error_message,
        }
    }
}
