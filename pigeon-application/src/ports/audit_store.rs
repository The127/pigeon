use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;

pub struct AuditEntry {
    pub command_name: &'static str,
    pub actor: String,
    pub org_id: OrganizationId,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait AuditStore: Send + Sync {
    async fn record(&self, entry: AuditEntry) -> Result<(), ApplicationError>;
}
