use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::ApplicationError;

pub struct AuditEntry {
    pub command_name: &'static str,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
}

#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait AuditStore: Send + Sync {
    async fn record(&self, entry: AuditEntry) -> Result<(), ApplicationError>;
}
