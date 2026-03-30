
use async_trait::async_trait;
use chrono::Utc;
use pigeon_domain::attempt::Attempt;
use pigeon_domain::dead_letter::{DeadLetter, DeadLetterId};
use pigeon_domain::event::DomainEvent;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;

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

#[derive(Default)]
pub struct ReplayDeadLetterHandler;

impl ReplayDeadLetterHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandHandler<ReplayDeadLetter> for ReplayDeadLetterHandler {
    async fn handle(
        &self,
        command: ReplayDeadLetter,
        ctx: &mut RequestContext,
    ) -> Result<DeadLetter, ApplicationError> {

        let mut dead_letter = ctx.uow()
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

        ctx.uow().dead_letter_store().save(&dead_letter).await?;
        ctx.uow().attempt_store().insert(&attempt).await?;
        ctx.uow().emit_event(DomainEvent::DeadLetterReplayed {
            dead_letter_id: dead_letter.id().clone(),
            message_id: dead_letter.message_id().clone(),
            endpoint_id: dead_letter.endpoint_id().clone(),
        });

        Ok(dead_letter)
    }
}
