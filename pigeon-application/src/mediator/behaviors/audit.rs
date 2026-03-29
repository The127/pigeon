use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::error::ApplicationError;
use crate::mediator::pipeline::{NextFn, PipelineBehavior, RequestContext};
use crate::ports::audit_store::{AuditEntry, AuditStore};

pub struct AuditBehavior {
    audit_store: Arc<dyn AuditStore>,
}

impl AuditBehavior {
    pub fn new(audit_store: Arc<dyn AuditStore>) -> Self {
        Self { audit_store }
    }
}

#[async_trait]
impl PipelineBehavior for AuditBehavior {
    async fn handle(
        &self,
        context: &mut RequestContext,
        next: NextFn,
    ) -> Result<(), ApplicationError> {
        let result = next().await;

        let (success, error_message) = match &result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };

        // Record audit for both success and failure — best effort, don't fail the request
        let _ = self
            .audit_store
            .record(AuditEntry {
                command_name: context.command_name,
                actor: context.actor.clone(),
                org_id: context.org_id.clone(),
                timestamp: Utc::now(),
                success,
                error_message,
            })
            .await;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeAuditStore, OperationLog};
    use pigeon_domain::organization::OrganizationId;

    fn make_ctx() -> RequestContext {
        RequestContext {
            command_name: "CreateApplication",
            actor: "user_42".into(),
            org_id: OrganizationId::new(),
        }
    }

    fn success_next() -> NextFn {
        Box::new(|| Box::pin(async { Ok(()) }))
    }

    fn failing_next() -> NextFn {
        Box::new(|| {
            Box::pin(async { Err(ApplicationError::Internal("boom".into())) })
        })
    }

    #[tokio::test]
    async fn records_audit_on_success() {
        let log = OperationLog::new();
        let audit_store = Arc::new(FakeAuditStore::new(log.clone()));
        let behavior = AuditBehavior::new(audit_store);

        let result = behavior.handle(&mut make_ctx(), success_next()).await;

        assert!(result.is_ok());
        assert_eq!(log.entries(), vec!["audit:record:CreateApplication:success"]);
    }

    #[tokio::test]
    async fn records_audit_on_failure() {
        let log = OperationLog::new();
        let audit_store = Arc::new(FakeAuditStore::new(log.clone()));
        let behavior = AuditBehavior::new(audit_store);

        let result = behavior.handle(&mut make_ctx(), failing_next()).await;

        assert!(result.is_err());
        assert_eq!(log.entries(), vec!["audit:record:CreateApplication:failure"]);
    }

    #[tokio::test]
    async fn propagates_handler_error() {
        let log = OperationLog::new();
        let audit_store = Arc::new(FakeAuditStore::new(log));
        let behavior = AuditBehavior::new(audit_store);

        let result = behavior.handle(&mut make_ctx(), failing_next()).await;

        assert!(matches!(result, Err(ApplicationError::Internal(_))));
    }
}
