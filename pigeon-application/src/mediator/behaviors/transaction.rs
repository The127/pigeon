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
        _context: &mut RequestContext,
        next: NextFn,
    ) -> Result<(), ApplicationError> {
        let uow = self.uow_factory.begin().await?;

        let result = next().await;

        match result {
            Ok(()) => {
                uow.commit().await?;
                Ok(())
            }
            Err(e) => {
                let _ = uow.rollback().await;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};

    fn make_ctx() -> RequestContext {
        RequestContext {
            command_name: "TestCommand",
            actor: "user_1".into(),
        }
    }

    fn success_next() -> NextFn {
        Box::new(|| Box::pin(async { Ok(()) }))
    }

    fn failing_next() -> NextFn {
        Box::new(|| {
            Box::pin(async { Err(ApplicationError::Internal("handler failed".into())) })
        })
    }

    #[tokio::test]
    async fn commits_on_success() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let behavior = TransactionBehavior::new(factory);

        let result = behavior.handle(&mut make_ctx(), success_next()).await;

        assert!(result.is_ok());
        assert_eq!(
            log.entries(),
            vec!["uow_factory:begin", "uow:commit"]
        );
    }

    #[tokio::test]
    async fn rolls_back_on_failure() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let behavior = TransactionBehavior::new(factory);

        let result = behavior.handle(&mut make_ctx(), failing_next()).await;

        assert!(result.is_err());
        assert_eq!(
            log.entries(),
            vec!["uow_factory:begin", "uow:rollback"]
        );
    }

    #[tokio::test]
    async fn propagates_handler_error() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
        let behavior = TransactionBehavior::new(factory);

        let result = behavior.handle(&mut make_ctx(), failing_next()).await;

        assert!(matches!(result, Err(ApplicationError::Internal(_))));
    }
}
