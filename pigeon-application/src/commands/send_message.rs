use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::attempt::Attempt;
use pigeon_domain::event::DomainEvent;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::message::Message;
use pigeon_domain::organization::OrganizationId;
use serde_json::Value;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;
use crate::ports::stores::EndpointReadStore;

#[derive(Debug)]
pub struct SendMessage {
    pub org_id: OrganizationId,
    pub app_id: ApplicationId,
    pub event_type_id: EventTypeId,
    pub payload: Value,
    pub idempotency_key: Option<String>,
}

impl Command for SendMessage {
    type Output = SendMessageResult;

    fn command_name(&self) -> &'static str {
        "SendMessage"
    }
}

#[derive(Debug)]
pub struct SendMessageResult {
    pub message: Message,
    pub attempts_created: usize,
    pub was_duplicate: bool,
}

pub struct SendMessageHandler {
    endpoint_read_store: Arc<dyn EndpointReadStore>,
    idempotency_ttl: Duration,
}

impl SendMessageHandler {
    pub fn new(
        endpoint_read_store: Arc<dyn EndpointReadStore>,
        idempotency_ttl: Duration,
    ) -> Self {
        Self {
            endpoint_read_store,
            idempotency_ttl,
        }
    }
}

#[async_trait]
impl CommandHandler<SendMessage> for SendMessageHandler {
    async fn handle(&self, command: SendMessage, ctx: &mut RequestContext) -> Result<SendMessageResult, ApplicationError> {
        let message = Message::new(
            command.app_id.clone(),
            command.event_type_id.clone(),
            command.payload,
            command.idempotency_key,
            self.idempotency_ttl,
        )
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;


        let result = ctx.uow().message_store().insert_or_get_existing(&message, &command.org_id).await?;

        if result.was_existing {
            return Ok(SendMessageResult {
                message: result.message,
                attempts_created: 0,
                was_duplicate: true,
            });
        }

        let endpoints = self
            .endpoint_read_store
            .find_enabled_by_app_and_event_type(
                &command.app_id,
                &command.event_type_id,
                &command.org_id,
            )
            .await?;

        let attempts_created = endpoints.len();

        for endpoint in &endpoints {
            let attempt = Attempt::new(
                result.message.id().clone(),
                endpoint.id().clone(),
                Utc::now(),
            );
            ctx.uow().attempt_store().insert(&attempt).await?;
        }

        ctx.uow().emit_event(DomainEvent::MessageCreated {
            message_id: result.message.id().clone(),
            app_id: command.app_id,
            event_type_id: command.event_type_id,
            attempts_created: attempts_created as u32,
        });


        Ok(SendMessageResult {
            message: result.message,
            attempts_created,
            was_duplicate: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mediator::pipeline::RequestContext;
    use crate::ports::unit_of_work::UnitOfWorkFactory;
    use crate::test_support::fakes::{
        FakeEndpointReadStore, FakeUnitOfWorkFactory, OperationLog, SharedMessageData,
    };
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::organization::OrganizationId;
    use serde_json::json;

    fn default_ttl() -> Duration {
        Duration::hours(24)
    }

    #[tokio::test]
    async fn new_message_fans_out_to_endpoints() {
        let log = OperationLog::new();
        let app_id = ApplicationId::new();
        let event_type_id = EventTypeId::new();

        let ep1 = Endpoint::new(
            app_id.clone(),
            None,
            "https://a.com/hook".into(),
            vec![event_type_id.clone()],
        )
        .unwrap();
        let ep2 = Endpoint::new(
            app_id.clone(),
            None,
            "https://b.com/hook".into(),
            vec![event_type_id.clone()],
        )
        .unwrap();

        let endpoint_store = Arc::new(FakeEndpointReadStore::new(
            log.clone(),
            vec![ep1, ep2],
        ));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = SendMessageHandler::new(endpoint_store, default_ttl());

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!({"user": "u1"}),
                idempotency_key: Some("key-1".into()),
            }, &mut ctx)
            .await
            .unwrap();

        assert_eq!(result.attempts_created, 2);
        assert!(!result.was_duplicate);
        assert_eq!(result.message.idempotency_key().as_str(), "key-1");
        assert!(log.entries().contains(&"attempt_store:insert".to_string()));
    }

    #[tokio::test]
    async fn duplicate_idempotency_key_returns_existing() {
        let log = OperationLog::new();
        let app_id = ApplicationId::new();
        let event_type_id = EventTypeId::new();

        let existing = Message::new(
            app_id.clone(),
            event_type_id.clone(),
            json!({"user": "u1"}),
            Some("dup-key".into()),
            default_ttl(),
        )
        .unwrap();

        let msg_data = SharedMessageData::default();
        msg_data.messages.lock().unwrap().push(existing.clone());

        let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), vec![]));
        let factory = Arc::new(FakeUnitOfWorkFactory::new_with_messages(
            log.clone(),
            msg_data,
        ));
        let handler = SendMessageHandler::new(endpoint_store, default_ttl());

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!({"user": "u1"}),
                idempotency_key: Some("dup-key".into()),
            }, &mut ctx)
            .await
            .unwrap();

        assert!(result.was_duplicate);
        assert_eq!(result.attempts_created, 0);
        assert!(!log.entries().contains(&"attempt_store:insert".to_string()));
    }

    #[tokio::test]
    async fn no_matching_endpoints_creates_zero_attempts() {
        let log = OperationLog::new();
        let app_id = ApplicationId::new();
        let event_type_id = EventTypeId::new();

        let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), vec![]));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = SendMessageHandler::new(endpoint_store, default_ttl());

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!({"data": true}),
                idempotency_key: None,
            }, &mut ctx)
            .await
            .unwrap();

        assert_eq!(result.attempts_created, 0);
        assert!(!result.was_duplicate);
    }

    #[tokio::test]
    async fn rejects_non_object_payload() {
        let log = OperationLog::new();
        let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), vec![]));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = SendMessageHandler::new(endpoint_store, default_ttl());

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id: ApplicationId::new(),
                event_type_id: EventTypeId::new(),
                payload: json!("not an object"),
                idempotency_key: None,
            }, &mut ctx)
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }

    #[tokio::test]
    async fn multiple_endpoints_create_one_attempt_each() {
        let log = OperationLog::new();
        let app_id = ApplicationId::new();
        let event_type_id = EventTypeId::new();

        let endpoints: Vec<Endpoint> = (0..4)
            .map(|i| {
                Endpoint::new(
                    app_id.clone(),
                    None,
                    format!("https://ep{i}.com/hook"),
                    vec![event_type_id.clone()],
                )
                .unwrap()
            })
            .collect();

        let endpoint_store =
            Arc::new(FakeEndpointReadStore::new(log.clone(), endpoints));
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let handler = SendMessageHandler::new(endpoint_store, default_ttl());

        let uow = factory.begin().await.unwrap();
        let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
        ctx.set_uow(uow);

        let result = handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!({"data": true}),
                idempotency_key: None,
            }, &mut ctx)
            .await
            .unwrap();

        assert_eq!(result.attempts_created, 4);
        let insert_count = log
            .entries()
            .iter()
            .filter(|e| *e == "attempt_store:insert")
            .count();
        assert_eq!(insert_count, 4);
    }
}
