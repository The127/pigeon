use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use pigeon_domain::attempt::{Attempt, AttemptId, AttemptStatus};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct RetryAttempt {
    pub org_id: OrganizationId,
    pub attempt_id: AttemptId,
}

impl Command for RetryAttempt {
    type Output = Attempt;

    fn command_name(&self) -> &'static str {
        "RetryAttempt"
    }
}

pub struct RetryAttemptHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl RetryAttemptHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<RetryAttempt> for RetryAttemptHandler {
    async fn handle(&self, command: RetryAttempt) -> Result<Attempt, ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let mut attempt = uow
            .attempt_store()
            .find_by_id(&command.attempt_id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if attempt.status() != AttemptStatus::Failed {
            return Err(ApplicationError::Validation(
                "Only failed attempts can be retried".to_string(),
            ));
        }

        attempt.mark_for_retry(Utc::now());

        uow.attempt_store().save(&attempt).await?;
        uow.commit().await?;

        Ok(attempt)
    }
}
