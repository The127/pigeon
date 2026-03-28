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

        if result.is_ok() {
            self.audit_store
                .record(AuditEntry {
                    command_name: context.command_name,
                    actor: context.actor.clone(),
                    timestamp: Utc::now(),
                })
                .await?;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeAuditStore, OperationLog};

    fn make_ctx() -> RequestContext {
        RequestContext {
            command_name: "CreateApplication",
            actor: "user_42".into(),
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
        assert_eq!(log.entries(), vec!["audit:record:CreateApplication"]);
    }

    #[tokio::test]
    async fn skips_audit_on_failure() {
        let log = OperationLog::new();
        let audit_store = Arc::new(FakeAuditStore::new(log.clone()));
        let behavior = AuditBehavior::new(audit_store);

        let result = behavior.handle(&mut make_ctx(), failing_next()).await;

        assert!(result.is_err());
        assert!(log.entries().is_empty());
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
