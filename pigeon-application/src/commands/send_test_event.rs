use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::attempt::Attempt;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::event_type::TEST_EVENT_TYPE_NAME;
use pigeon_domain::message::Message;
use pigeon_domain::organization::OrganizationId;
use serde_json::json;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::stores::EventTypeReadStore;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct SendTestEvent {
    pub org_id: OrganizationId,
    pub app_id: ApplicationId,
    pub endpoint_id: EndpointId,
}

impl Command for SendTestEvent {
    type Output = SendTestEventResult;

    fn command_name(&self) -> &'static str {
        "SendTestEvent"
    }
}

#[derive(Debug)]
pub struct SendTestEventResult {
    pub message: Message,
}

pub struct SendTestEventHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
    event_type_read_store: Arc<dyn EventTypeReadStore>,
}

impl SendTestEventHandler {
    pub fn new(
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        event_type_read_store: Arc<dyn EventTypeReadStore>,
    ) -> Self {
        Self {
            uow_factory,
            event_type_read_store,
        }
    }
}

#[async_trait]
impl CommandHandler<SendTestEvent> for SendTestEventHandler {
    async fn handle(
        &self,
        command: SendTestEvent,
    ) -> Result<SendTestEventResult, ApplicationError> {
        let test_event_type = self
            .event_type_read_store
            .find_by_app_and_name(&command.app_id, TEST_EVENT_TYPE_NAME, &command.org_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::Internal(
                    "pigeon.test event type not found — application may have been created before this feature".to_string(),
                )
            })?;

        let payload = json!({
            "type": "pigeon.test",
            "timestamp": Utc::now().to_rfc3339(),
            "message": "This is a test event from Pigeon"
        });

        let message = Message::new(
            command.app_id,
            test_event_type.id().clone(),
            payload,
            None,
            Duration::hours(1),
        )
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let attempt = Attempt::new(
            message.id().clone(),
            command.endpoint_id,
            Utc::now(),
        );

        let mut uow = self.uow_factory.begin().await?;
        uow.message_store()
            .insert_or_get_existing(&message, &command.org_id)
            .await?;
        uow.attempt_store().insert(&attempt).await?;
        uow.commit().await?;

        Ok(SendTestEventResult { message })
    }
}
