use std::sync::Arc;

use async_trait::async_trait;

use crate::error::ApplicationError;
use crate::mediator::pipeline::{NextFn, PipelineBehavior, RequestContext};
use crate::ports::unit_of_work::UnitOfWorkFactory;

pub struct TransactionBehavior {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl TransactionBehavior {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl PipelineBehavior for TransactionBehavior {
    async fn handle(
        &self,
        mut context: RequestContext,
        next: NextFn,
    ) -> (RequestContext, Result<(), ApplicationError>) {
        let uow = match self.uow_factory.begin().await {
            Ok(uow) => uow,
            Err(e) => return (context, Err(e)),
        };
        context.set_uow(uow);

        let (mut context, result) = next(context).await;

        let uow = match context.take_uow() {
            Some(uow) => uow,
            None => {
                return (
                    context,
                    Err(ApplicationError::Internal("UoW missing after handler".into())),
                )
            }
        };

        match result {
            Ok(()) => {
                let commit_result = uow.commit().await;
                (context, commit_result)
            }
            Err(e) => {
                let _ = uow.rollback().await;
                (context, Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::organization::OrganizationId;

    fn make_ctx() -> RequestContext {
        RequestContext::new("TestCommand", "user_1".into(), OrganizationId::new())
    }

    fn success_next() -> NextFn {
        Box::new(|ctx: RequestContext| Box::pin(async { (ctx, Ok(())) }))
    }

    fn failing_next() -> NextFn {
        Box::new(|ctx: RequestContext| {
            Box::pin(async { (ctx, Err(ApplicationError::Internal("handler failed".into()))) })
        })
    }

    #[tokio::test]
    async fn commits_on_success() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let behavior = TransactionBehavior::new(factory);

        let (_, result) = behavior.handle(make_ctx(), success_next()).await;

        assert!(result.is_ok());
        assert_eq!(log.entries(), vec!["uow_factory:begin", "uow:commit"]);
    }

    #[tokio::test]
    async fn rolls_back_on_failure() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let behavior = TransactionBehavior::new(factory);

        let (_, result) = behavior.handle(make_ctx(), failing_next()).await;

        assert!(result.is_err());
        assert_eq!(log.entries(), vec!["uow_factory:begin", "uow:rollback"]);
    }

    #[tokio::test]
    async fn propagates_handler_error() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
        let behavior = TransactionBehavior::new(factory);

        let (_, result) = behavior.handle(make_ctx(), failing_next()).await;

        assert!(matches!(result, Err(ApplicationError::Internal(_))));
    }

    #[tokio::test]
    async fn handler_can_access_uow_from_context() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let behavior = TransactionBehavior::new(factory);

        let handler: NextFn = Box::new(|mut ctx: RequestContext| {
            Box::pin(async move {
                let _store = ctx.uow().application_store();
                (ctx, Ok(()))
            })
        });

        let (_, result) = behavior.handle(make_ctx(), handler).await;

        assert!(result.is_ok());
        assert_eq!(log.entries(), vec!["uow_factory:begin", "uow:commit"]);
    }
}
