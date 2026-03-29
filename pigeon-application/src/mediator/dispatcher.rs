use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::audit_store::{AuditEntry, AuditStore};

/// Dispatches a command through a handler with automatic audit logging.
///
/// Records an audit entry after every execution (success or failure).
/// The actor and org_id come from the caller (API handler has AuthContext).
/// Audit recording is best-effort — failures are logged but don't fail the request.
pub async fn dispatch<C: Command>(
    handler: &dyn CommandHandler<C>,
    command: C,
    actor: &str,
    org_id: &OrganizationId,
    audit_store: &dyn AuditStore,
) -> Result<C::Output, ApplicationError> {
    let command_name = command.command_name();
    let result = handler.handle(command).await;

    let (success, error_message) = match &result {
        Ok(_) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };

    let _ = audit_store
        .record(AuditEntry {
            command_name,
            actor: actor.to_string(),
            org_id: org_id.clone(),
            timestamp: chrono::Utc::now(),
            success,
            error_message,
        })
        .await;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::test_support::fakes::{FakeAuditStore, FakeUnitOfWorkFactory, OperationLog};
    use crate::commands::create_application::{CreateApplication, CreateApplicationHandler};

    #[tokio::test]
    async fn dispatch_records_audit_on_success() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateApplicationHandler::new(factory);
        let audit = FakeAuditStore::new(log.clone());
        let org_id = OrganizationId::new();

        let result = dispatch(
            &handler,
            CreateApplication {
                org_id: org_id.clone(),
                name: "test-app".into(),
                uid: "test-app".into(),
            },
            "user_1",
            &org_id,
            &audit,
        )
        .await;

        assert!(result.is_ok());
        assert!(log.entries().iter().any(|e| e.ends_with(":success")));
    }

    #[tokio::test]
    async fn dispatch_records_audit_on_failure() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = CreateApplicationHandler::new(factory);
        let audit = FakeAuditStore::new(log.clone());
        let org_id = OrganizationId::new();

        let result = dispatch(
            &handler,
            CreateApplication {
                org_id: org_id.clone(),
                name: "".into(), // will fail validation
                uid: "test".into(),
            },
            "user_1",
            &org_id,
            &audit,
        )
        .await;

        assert!(result.is_err());
        assert!(log.entries().iter().any(|e| e.ends_with(":failure")));
    }
}
