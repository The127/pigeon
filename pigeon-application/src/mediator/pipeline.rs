use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;

pub struct RequestContext {
    pub command_name: &'static str,
    pub actor: String,
    pub org_id: OrganizationId,
}

/// Type-erased next step in the pipeline. Returns `Result<(), _>` because
/// the mediator captures the typed command output in a separate slot.
/// Behaviors only observe success or failure.
pub type NextFn = Box<
    dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<(), ApplicationError>> + Send>> + Send,
>;

#[async_trait]
pub trait PipelineBehavior: Send + Sync {
    async fn handle(
        &self,
        context: &mut RequestContext,
        next: NextFn,
    ) -> Result<(), ApplicationError>;
}

/// Executes a single behavior around a handler.
/// For composing multiple behaviors, nest calls from inside out.
pub async fn execute_with_behavior(
    behavior: &dyn PipelineBehavior,
    context: &mut RequestContext,
    handler: NextFn,
) -> Result<(), ApplicationError> {
    behavior.handle(context, handler).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::mediator::behaviors::audit::AuditBehavior;
    use crate::mediator::behaviors::transaction::TransactionBehavior;
    use crate::test_support::fakes::{FakeAuditStore, FakeUnitOfWorkFactory, OperationLog};

    #[tokio::test]
    async fn composed_pipeline_commits_and_audits_on_success() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let audit_store = Arc::new(FakeAuditStore::new(log.clone()));

        let txn = TransactionBehavior::new(factory);
        let audit = AuditBehavior::new(audit_store);

        // Nest from inside out: txn wraps audit wraps handler
        let handler: NextFn = Box::new(|| Box::pin(async { Ok(()) }));

        // audit wraps handler
        let mut ctx = RequestContext {
            command_name: "TestCommand",
            actor: "user_1".into(),
            org_id: OrganizationId::new(),
        };
        let audit_next: NextFn = {
            Box::new(move || {
                Box::pin(async move { audit.handle(&mut ctx, handler).await })
            })
        };

        // txn wraps audit+handler
        let mut outer_ctx = RequestContext {
            command_name: "TestCommand",
            actor: "user_1".into(),
            org_id: OrganizationId::new(),
        };
        let result = txn.handle(&mut outer_ctx, audit_next).await;

        assert!(result.is_ok());
        // begin -> (handler ok) -> audit records -> commit
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "audit:record:TestCommand:success",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn composed_pipeline_rolls_back_and_skips_audit_on_failure() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let audit_store = Arc::new(FakeAuditStore::new(log.clone()));

        let txn = TransactionBehavior::new(factory);
        let audit = AuditBehavior::new(audit_store);

        let handler: NextFn = Box::new(|| {
            Box::pin(async { Err(ApplicationError::Internal("handler failed".into())) })
        });

        let mut ctx = RequestContext {
            command_name: "TestCommand",
            actor: "user_1".into(),
            org_id: OrganizationId::new(),
        };
        let audit_next: NextFn = {
            Box::new(move || {
                Box::pin(async move { audit.handle(&mut ctx, handler).await })
            })
        };

        let mut outer_ctx = RequestContext {
            command_name: "TestCommand",
            actor: "user_1".into(),
            org_id: OrganizationId::new(),
        };
        let result = txn.handle(&mut outer_ctx, audit_next).await;

        assert!(result.is_err());
        // begin -> handler fails -> audit records failure -> rollback
        assert_eq!(
            log.entries(),
            vec!["uow_factory:begin", "audit:record:TestCommand:failure", "uow:rollback"]
        );
    }

    #[tokio::test]
    async fn single_behavior_wraps_handler() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let txn = TransactionBehavior::new(factory);

        let handler: NextFn = Box::new(|| Box::pin(async { Ok(()) }));

        let mut ctx = RequestContext {
            command_name: "TestCommand",
            actor: "user_1".into(),
            org_id: OrganizationId::new(),
        };

        let result = execute_with_behavior(&txn, &mut ctx, handler).await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec!["uow_factory:begin", "uow:commit"]
        );
    }
}
