use std::sync::Arc;

use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::behaviors::audit::AuditBehavior;
use crate::mediator::behaviors::transaction::TransactionBehavior;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::{PipelineBehavior, RequestContext};
use crate::ports::audit_store::AuditStore;
use crate::ports::unit_of_work::UnitOfWorkFactory;

/// Dispatches a command through the full pipeline:
/// TransactionBehavior → AuditBehavior → CommandHandler
///
/// - TransactionBehavior creates the UoW, stores it in context, commits/rolls back
/// - AuditBehavior records success/failure after handler execution (best-effort)
/// - CommandHandler receives the command + context, uses ctx.uow() for store operations
///
/// The handler's typed output is stored in RequestContext via set_output/take_output
/// (type-erased with Box<dyn Any>) since the pipeline returns Result<(), _>.
pub async fn dispatch<C: Command>(
    handler: Arc<dyn CommandHandler<C>>,
    command: C,
    actor: &str,
    org_id: &OrganizationId,
    uow_factory: Arc<dyn UnitOfWorkFactory>,
    audit_store: Arc<dyn AuditStore>,
) -> Result<C::Output, ApplicationError> {
    let command_name = command.command_name();
    let txn = TransactionBehavior::new(uow_factory);
    let audit = AuditBehavior::new(audit_store);

    let ctx = RequestContext::new(command_name, actor.to_string(), org_id.clone());

    // Pipeline: txn wraps audit wraps handler
    // Ownership of ctx flows: txn → audit → handler → audit → txn → dispatch
    let (mut ctx, result) = txn
        .handle(
            ctx,
            Box::new(move |ctx: RequestContext| {
                Box::pin(async move {
                    audit
                        .handle(
                            ctx,
                            Box::new(move |mut ctx: RequestContext| {
                                Box::pin(async move {
                                    match handler.handle(command, &mut ctx).await {
                                        Ok(output) => {
                                            ctx.set_output(output);
                                            (ctx, Ok(()))
                                        }
                                        Err(e) => (ctx, Err(e)),
                                    }
                                })
                            }),
                        )
                        .await
                })
            }),
        )
        .await;

    result?;

    ctx.take_output::<C::Output>().ok_or_else(|| {
        ApplicationError::Internal("handler completed successfully but produced no output".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::create_application::{CreateApplication, CreateApplicationHandler};
    use crate::test_support::fakes::{FakeAuditStore, FakeUnitOfWorkFactory, OperationLog};

    #[tokio::test]
    async fn dispatch_records_audit_on_success() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = Arc::new(CreateApplicationHandler::new());
        let audit = Arc::new(FakeAuditStore::new(log.clone()));
        let org_id = OrganizationId::new();

        let result = dispatch(
            handler,
            CreateApplication {
                org_id: org_id.clone(),
                name: "test-app".into(),
                uid: "test-app".into(),
            },
            "user_1",
            &org_id,
            factory,
            audit,
        )
        .await;

        assert!(result.is_ok());
        let entries = log.entries();
        assert!(entries.contains(&"uow_factory:begin".to_string()));
        assert!(entries.contains(&"uow:commit".to_string()));
        assert!(entries.iter().any(|e| e.ends_with(":success")));
    }

    #[tokio::test]
    async fn dispatch_records_audit_on_failure() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = Arc::new(CreateApplicationHandler::new());
        let audit = Arc::new(FakeAuditStore::new(log.clone()));
        let org_id = OrganizationId::new();

        let result = dispatch(
            handler,
            CreateApplication {
                org_id: org_id.clone(),
                name: "".into(), // will fail validation
                uid: "test".into(),
            },
            "user_1",
            &org_id,
            factory,
            audit,
        )
        .await;

        assert!(result.is_err());
        let entries = log.entries();
        assert!(entries.contains(&"uow_factory:begin".to_string()));
        assert!(entries.contains(&"uow:rollback".to_string()));
        assert!(entries.iter().any(|e| e.ends_with(":failure")));
    }
}
