use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use pigeon_domain::attempt::Attempt;
use pigeon_domain::event::DomainEvent;
use pigeon_domain::message::{Message, MessageId};
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::stores::{AttemptReadStore, EndpointReadStore, MessageReadStore};
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct RetriggerMessage {
    pub message_id: MessageId,
    pub org_id: OrganizationId,
}

impl Command for RetriggerMessage {
    type Output = RetriggerMessageResult;

    fn command_name(&self) -> &'static str {
        "RetriggerMessage"
    }
}

#[derive(Debug)]
pub struct RetriggerMessageResult {
    pub message: Message,
    pub attempts_created: usize,
}

pub struct RetriggerMessageHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
    message_read_store: Arc<dyn MessageReadStore>,
    endpoint_read_store: Arc<dyn EndpointReadStore>,
    attempt_read_store: Arc<dyn AttemptReadStore>,
}

impl RetriggerMessageHandler {
    pub fn new(
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        message_read_store: Arc<dyn MessageReadStore>,
        endpoint_read_store: Arc<dyn EndpointReadStore>,
        attempt_read_store: Arc<dyn AttemptReadStore>,
    ) -> Self {
        Self {
            uow_factory,
            message_read_store,
            endpoint_read_store,
            attempt_read_store,
        }
    }
}

#[async_trait]
impl CommandHandler<RetriggerMessage> for RetriggerMessageHandler {
    async fn handle(
        &self,
        command: RetriggerMessage,
    ) -> Result<RetriggerMessageResult, ApplicationError> {
        let message_with_status = self
            .message_read_store
            .find_by_id(&command.message_id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        let message = message_with_status.message;

        let all_endpoints = self
            .endpoint_read_store
            .find_enabled_by_app_and_event_type(
                message.app_id(),
                message.event_type_id(),
                &command.org_id,
            )
            .await?;

        // Filter out endpoints that already have attempts for this message
        let existing_attempts = self
            .attempt_read_store
            .list_by_message(message.id(), &command.org_id)
            .await?;
        let existing_endpoint_ids: std::collections::HashSet<_> = existing_attempts
            .iter()
            .map(|a| a.endpoint_id().clone())
            .collect();

        let new_endpoints: Vec<_> = all_endpoints
            .into_iter()
            .filter(|ep| !existing_endpoint_ids.contains(ep.id()))
            .collect();

        if new_endpoints.is_empty() {
            return Err(ApplicationError::Validation(
                "no new endpoints to deliver to — all matching endpoints already have attempts".to_string(),
            ));
        }

        let attempts_created = new_endpoints.len();

        let mut uow = self.uow_factory.begin().await?;
        for endpoint in &new_endpoints {
            let attempt = Attempt::new(
                message.id().clone(),
                endpoint.id().clone(),
                Utc::now(),
            );
            uow.attempt_store().insert(&attempt).await?;
        }
        uow.emit_event(DomainEvent::MessageRetriggered {
            message_id: message.id().clone(),
            attempts_created: attempts_created as u32,
        });
        uow.commit().await?;

        Ok(RetriggerMessageResult {
            message,
            attempts_created,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::message_status::MessageWithStatus;
    use crate::ports::stores::{MockAttemptReadStore, MockMessageReadStore};
    use crate::test_support::fakes::{
        FakeEndpointReadStore, FakeUnitOfWorkFactory, OperationLog,
    };
    use pigeon_domain::attempt::AttemptState;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::message::MessageState;
    use pigeon_domain::organization::OrganizationId;

    fn wrap(msg: Message) -> MessageWithStatus {
        MessageWithStatus { message: msg, attempts_created: 0, succeeded: 0, failed: 0, dead_lettered: 0 }
    }

    fn empty_attempt_store() -> Arc<MockAttemptReadStore> {
        let mut mock = MockAttemptReadStore::new();
        mock.expect_list_by_message().returning(|_, _| Ok(vec![]));
        Arc::new(mock)
    }

    #[tokio::test]
    async fn retriggers_to_matching_endpoints() {
        let log = OperationLog::new();
        let msg = Message::reconstitute(MessageState::fake());
        let msg_clone = msg.clone();
        let app_id = msg.app_id().clone();
        let event_type_id = msg.event_type_id().clone();
        let message_id = msg.id().clone();

        let mut msg_store = MockMessageReadStore::new();
        msg_store
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(wrap(msg_clone.clone()))));

        let ep = Endpoint::new(
            app_id.clone(),
            None,
            "https://a.com/hook".into(),
            "whsec_a".into(),
            vec![event_type_id.clone()],
        )
        .unwrap();

        let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), vec![ep]));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));

        let handler = RetriggerMessageHandler::new(
            factory,
            Arc::new(msg_store),
            endpoint_store,
            empty_attempt_store(),
        );

        let result = handler
            .handle(RetriggerMessage {
                message_id,
                org_id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert_eq!(result.attempts_created, 1);
        assert!(log.entries().contains(&"attempt_store:insert".to_string()));
    }

    #[tokio::test]
    async fn returns_not_found_for_missing_message() {
        let log = OperationLog::new();
        let mut msg_store = MockMessageReadStore::new();
        msg_store
            .expect_find_by_id()
            .returning(|_, _| Ok(None));

        let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), vec![]));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));

        let handler = RetriggerMessageHandler::new(
            factory,
            Arc::new(msg_store),
            endpoint_store,
            empty_attempt_store(),
        );

        let result = handler
            .handle(RetriggerMessage {
                message_id: MessageId::new(),
                org_id: OrganizationId::new(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn returns_validation_error_when_no_endpoints_match() {
        let log = OperationLog::new();
        let msg = Message::reconstitute(MessageState::fake());
        let msg_clone = msg.clone();
        let message_id = msg.id().clone();

        let mut msg_store = MockMessageReadStore::new();
        msg_store
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(wrap(msg_clone.clone()))));

        let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), vec![]));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));

        let handler = RetriggerMessageHandler::new(
            factory,
            Arc::new(msg_store),
            endpoint_store,
            empty_attempt_store(),
        );

        let result = handler
            .handle(RetriggerMessage {
                message_id,
                org_id: OrganizationId::new(),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
        // No UoW should have been started
        assert!(!log.entries().contains(&"uow_factory:begin".to_string()));
    }

    #[tokio::test]
    async fn fans_out_to_multiple_endpoints() {
        let log = OperationLog::new();
        let msg = Message::reconstitute(MessageState::fake());
        let msg_clone = msg.clone();
        let app_id = msg.app_id().clone();
        let event_type_id = msg.event_type_id().clone();
        let message_id = msg.id().clone();

        let mut msg_store = MockMessageReadStore::new();
        msg_store
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(wrap(msg_clone.clone()))));

        let endpoints: Vec<Endpoint> = (0..3)
            .map(|i| {
                Endpoint::new(
                    app_id.clone(),
                    None,
                    format!("https://ep{i}.com/hook"),
                    format!("whsec_{i}"),
                    vec![event_type_id.clone()],
                )
                .unwrap()
            })
            .collect();

        let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), endpoints));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));

        let handler = RetriggerMessageHandler::new(
            factory,
            Arc::new(msg_store),
            endpoint_store,
            empty_attempt_store(),
        );

        let result = handler
            .handle(RetriggerMessage {
                message_id,
                org_id: OrganizationId::new(),
            })
            .await
            .unwrap();

        assert_eq!(result.attempts_created, 3);
        let insert_count = log
            .entries()
            .iter()
            .filter(|e| *e == "attempt_store:insert")
            .count();
        assert_eq!(insert_count, 3);
    }

    #[tokio::test]
    async fn skips_endpoints_that_already_have_attempts() {
        let log = OperationLog::new();
        let msg = Message::reconstitute(MessageState::fake());
        let msg_clone = msg.clone();
        let app_id = msg.app_id().clone();
        let event_type_id = msg.event_type_id().clone();
        let message_id = msg.id().clone();

        let mut msg_store = MockMessageReadStore::new();
        msg_store
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(wrap(msg_clone.clone()))));

        let ep = Endpoint::new(
            app_id.clone(),
            None,
            "https://a.com/hook".into(),
            "whsec_a".into(),
            vec![event_type_id.clone()],
        )
        .unwrap();
        let ep_id = ep.id().clone();

        // Simulate an existing attempt for this endpoint
        let existing_attempt = Attempt::reconstitute(AttemptState {
            id: pigeon_domain::attempt::AttemptId::new(),
            message_id: message_id.clone(),
            endpoint_id: ep_id,
            status: pigeon_domain::attempt::AttemptStatus::Succeeded,
            response_code: Some(200),
            response_body: None,
            attempted_at: Some(Utc::now()),
            next_attempt_at: None,
            attempt_number: 1,
            duration_ms: Some(50),
            version: pigeon_domain::version::Version::new(0),
        });

        let mut att_store = MockAttemptReadStore::new();
        att_store
            .expect_list_by_message()
            .returning(move |_, _| Ok(vec![existing_attempt.clone()]));

        let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), vec![ep]));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));

        let handler = RetriggerMessageHandler::new(
            factory,
            Arc::new(msg_store),
            endpoint_store,
            Arc::new(att_store),
        );

        let result = handler
            .handle(RetriggerMessage {
                message_id,
                org_id: OrganizationId::new(),
            })
            .await;

        // Should fail because the only matching endpoint already has an attempt
        assert!(matches!(result, Err(ApplicationError::Validation(_))));
        // No UoW should have been started
        assert!(!log.entries().contains(&"uow_factory:begin".to_string()));
    }
}
