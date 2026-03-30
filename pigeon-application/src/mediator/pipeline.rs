use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::ports::unit_of_work::UnitOfWork;

/// Request-scoped context threaded through the pipeline.
///
/// Created by `dispatch()`, populated by behaviors (TransactionBehavior sets the UoW),
/// and consumed by command handlers (via `ctx.uow()`).
///
/// Ownership is passed through the pipeline — each layer takes the context,
/// uses it, and returns it alongside the result. This avoids mutable borrow
/// conflicts between behaviors that need to read context after calling next.
pub struct RequestContext {
    pub command_name: &'static str,
    pub actor: String,
    pub org_id: OrganizationId,
    uow: Option<Box<dyn UnitOfWork>>,
    output: Option<Box<dyn Any + Send>>,
}

impl RequestContext {
    pub fn new(command_name: &'static str, actor: String, org_id: OrganizationId) -> Self {
        Self {
            command_name,
            actor,
            org_id,
            uow: None,
            output: None,
        }
    }

    /// Returns a mutable reference to the UoW managed by TransactionBehavior.
    ///
    /// # Panics
    /// Panics if called outside a TransactionBehavior pipeline — this is a programming
    /// error, not a runtime condition, so a panic with a clear message is appropriate.
    pub fn uow(&mut self) -> &mut dyn UnitOfWork {
        self.uow
            .as_deref_mut()
            .expect("UoW not available — handler called outside TransactionBehavior pipeline")
    }

    /// Set the UoW. Called by TransactionBehavior after begin().
    ///
    /// Also available to integration tests via the `test-support` feature.
    #[cfg(not(feature = "test-support"))]
    pub(crate) fn set_uow(&mut self, uow: Box<dyn UnitOfWork>) {
        self.uow = Some(uow);
    }

    /// Set the UoW. Called by TransactionBehavior after begin().
    #[cfg(feature = "test-support")]
    pub fn set_uow(&mut self, uow: Box<dyn UnitOfWork>) {
        self.uow = Some(uow);
    }

    /// Take the UoW for commit/rollback. Called by TransactionBehavior after handler returns.
    pub(crate) fn take_uow(&mut self) -> Option<Box<dyn UnitOfWork>> {
        self.uow.take()
    }

    /// Store the handler's typed output (type-erased). Called by the handler closure in dispatch().
    pub(crate) fn set_output<T: Any + Send>(&mut self, value: T) {
        self.output = Some(Box::new(value));
    }

    /// Extract the handler's typed output. Called by dispatch() after the pipeline completes.
    pub(crate) fn take_output<T: Any + Send>(&mut self) -> Option<T> {
        let boxed = self.output.take()?;
        boxed.downcast::<T>().ok().map(|b| *b)
    }
}

/// Next step in the pipeline. Takes ownership of RequestContext, returns it alongside
/// the result. Ownership transfer avoids mutable borrow conflicts — behaviors can read
/// context fields after calling next without fighting the borrow checker.
pub type NextFn = Box<
    dyn FnOnce(
            RequestContext,
        ) -> Pin<Box<dyn Future<Output = (RequestContext, Result<(), ApplicationError>)> + Send>>
        + Send,
>;

#[async_trait]
pub trait PipelineBehavior: Send + Sync {
    async fn handle(
        &self,
        context: RequestContext,
        next: NextFn,
    ) -> (RequestContext, Result<(), ApplicationError>);
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

        let ctx = RequestContext::new(
            "TestCommand",
            "user_1".into(),
            OrganizationId::new(),
        );

        let (_, result) = txn
            .handle(
                ctx,
                Box::new(move |ctx: RequestContext| {
                    Box::pin(async move {
                        audit
                            .handle(
                                ctx,
                                Box::new(|ctx: RequestContext| {
                                    Box::pin(async { (ctx, Ok(())) })
                                }),
                            )
                            .await
                    })
                }),
            )
            .await;

        assert!(result.is_ok());
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
    async fn composed_pipeline_rolls_back_and_audits_on_failure() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let audit_store = Arc::new(FakeAuditStore::new(log.clone()));

        let txn = TransactionBehavior::new(factory);
        let audit = AuditBehavior::new(audit_store);

        let ctx = RequestContext::new(
            "TestCommand",
            "user_1".into(),
            OrganizationId::new(),
        );

        let (_, result) = txn
            .handle(
                ctx,
                Box::new(move |ctx: RequestContext| {
                    Box::pin(async move {
                        audit
                            .handle(
                                ctx,
                                Box::new(|ctx: RequestContext| {
                                    Box::pin(async {
                                        (ctx, Err(ApplicationError::Internal("handler failed".into())))
                                    })
                                }),
                            )
                            .await
                    })
                }),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "audit:record:TestCommand:failure",
                "uow:rollback",
            ]
        );
    }
}
