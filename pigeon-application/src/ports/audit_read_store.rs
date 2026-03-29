use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    pub id: uuid::Uuid,
    pub command_name: String,
    pub actor: String,
    pub org_id: OrganizationId,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait AuditReadStore: Send + Sync {
    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
        command_filter: Option<String>,
        success_filter: Option<bool>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<AuditLogEntry>, ApplicationError>;
    async fn count_by_org(
        &self,
        org_id: &OrganizationId,
        command_filter: Option<String>,
        success_filter: Option<bool>,
    ) -> Result<u64, ApplicationError>;
}
