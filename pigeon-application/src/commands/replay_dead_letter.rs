use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use pigeon_domain::attempt::Attempt;
use pigeon_domain::dead_letter::{DeadLetter, DeadLetterId};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct ReplayDeadLetter {
    pub org_id: OrganizationId,
    pub dead_letter_id: DeadLetterId,
}

impl Command for ReplayDeadLetter {
    type Output = DeadLetter;

    fn command_name(&self) -> &'static str {
        "ReplayDeadLetter"
    }
}

pub struct ReplayDeadLetterHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl ReplayDeadLetterHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<ReplayDeadLetter> for ReplayDeadLetterHandler {
    async fn handle(
        &self,
        command: ReplayDeadLetter,
    ) -> Result<DeadLetter, ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let mut dead_letter = uow
            .dead_letter_store()
            .find_by_id(&command.dead_letter_id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if dead_letter.replayed_at().is_some() {
            return Err(ApplicationError::Validation(
                "Dead letter has already been replayed".to_string(),
            ));
        }

        dead_letter.mark_replayed();

        let attempt = Attempt::new(
            dead_letter.message_id().clone(),
            dead_letter.endpoint_id().clone(),
            Utc::now(),
        );

        uow.dead_letter_store().save(&dead_letter).await?;
        uow.attempt_store().insert(&attempt).await?;
        uow.commit().await?;

        Ok(dead_letter)
    }
}
