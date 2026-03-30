
use async_trait::async_trait;
use chrono::Utc;
use pigeon_domain::attempt::{Attempt, AttemptId, AttemptStatus};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

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

#[derive(Default)]
pub struct RetryAttemptHandler;

impl RetryAttemptHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<RetryAttempt> for RetryAttemptHandler {
    async fn handle(&self, command: RetryAttempt, ctx: &mut RequestContext) -> Result<Attempt, ApplicationError> {

        let mut attempt = ctx.uow()
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

        ctx.uow().attempt_store().save(&attempt).await?;

        Ok(attempt)
    }
}
