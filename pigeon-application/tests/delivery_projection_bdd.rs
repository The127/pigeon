use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cucumber::{given, then, when, World};

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::projection_store::ProjectionStore;
use pigeon_application::services::delivery_projection::DeliveryProjectionSubscriber;
use pigeon_application::services::outbox_worker::EventSubscriber;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::attempt::AttemptId;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::event::DomainEvent;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::message::MessageId;

// --- In-memory projection store for testing ---

#[derive(Debug, Clone, Default)]
struct EndpointSummary {
    total_success: i64,
    total_failure: i64,
    consecutive_failures: i64,
}

#[derive(Debug, Clone, Default)]
struct MessageStatus {
    attempts_created: i32,
    succeeded: i32,
    failed: i32,
    dead_lettered: i32,
}

#[derive(Default, Clone, Debug)]
struct FakeProjectionData {
    endpoints: Arc<Mutex<HashMap<EndpointId, EndpointSummary>>>,
    messages: Arc<Mutex<HashMap<MessageId, MessageStatus>>>,
}

struct FakeProjectionStore {
    data: FakeProjectionData,
}

#[async_trait]
impl ProjectionStore for FakeProjectionStore {
    async fn record_endpoint_success(
        &self,
        endpoint_id: &EndpointId,
        _delivered_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ApplicationError> {
        let mut eps = self.data.endpoints.lock().unwrap();
        let entry = eps.entry(endpoint_id.clone()).or_default();
        entry.total_success += 1;
        entry.consecutive_failures = 0;
        Ok(())
    }

    async fn record_endpoint_failure(
        &self,
        endpoint_id: &EndpointId,
        _delivered_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ApplicationError> {
        let mut eps = self.data.endpoints.lock().unwrap();
        let entry = eps.entry(endpoint_id.clone()).or_default();
        entry.total_failure += 1;
        entry.consecutive_failures += 1;
        Ok(())
    }

    async fn init_message_status(
        &self,
        message_id: &MessageId,
        attempts_created: u32,
    ) -> Result<(), ApplicationError> {
        let mut msgs = self.data.messages.lock().unwrap();
        msgs.insert(
            message_id.clone(),
            MessageStatus {
                attempts_created: attempts_created as i32,
                ..Default::default()
            },
        );
        Ok(())
    }

    async fn increment_message_succeeded(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError> {
        let mut msgs = self.data.messages.lock().unwrap();
        if let Some(s) = msgs.get_mut(message_id) {
            s.succeeded += 1;
        }
        Ok(())
    }

    async fn increment_message_failed(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError> {
        let mut msgs = self.data.messages.lock().unwrap();
        if let Some(s) = msgs.get_mut(message_id) {
            s.failed += 1;
        }
        Ok(())
    }

    async fn increment_message_dead_lettered(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError> {
        let mut msgs = self.data.messages.lock().unwrap();
        if let Some(s) = msgs.get_mut(message_id) {
            s.dead_lettered += 1;
        }
        Ok(())
    }
}

// --- World ---

#[derive(Debug, Default, World)]
pub struct ProjectionWorld {
    message_id: Option<MessageId>,
    endpoint_id: Option<EndpointId>,
    data: Option<FakeProjectionData>,
}

fn create_subscriber(data: &FakeProjectionData) -> DeliveryProjectionSubscriber {
    DeliveryProjectionSubscriber::new(Arc::new(FakeProjectionStore { data: data.clone() }))
}

// --- Given ---

#[given("a message delivery status exists")]
async fn given_message_status(world: &mut ProjectionWorld) {
    let data = FakeProjectionData::default();
    let message_id = MessageId::new();
    let endpoint_id = EndpointId::new();
    data.messages.lock().unwrap().insert(
        message_id.clone(),
        MessageStatus {
            attempts_created: 1,
            ..Default::default()
        },
    );
    world.message_id = Some(message_id);
    world.endpoint_id = Some(endpoint_id);
    world.data = Some(data);
}

#[given("an endpoint with 3 consecutive failures in the summary")]
async fn given_endpoint_with_failures(world: &mut ProjectionWorld) {
    let data = FakeProjectionData::default();
    let endpoint_id = EndpointId::new();
    data.endpoints.lock().unwrap().insert(
        endpoint_id.clone(),
        EndpointSummary {
            total_failure: 3,
            consecutive_failures: 3,
            ..Default::default()
        },
    );
    world.endpoint_id = Some(endpoint_id);
    world.message_id = Some(MessageId::new());
    world.data = Some(data);
}

// --- When ---

#[when(regex = r"a MessageCreated event with (\d+) attempts is processed")]
async fn when_message_created(world: &mut ProjectionWorld, attempts: u32) {
    let data = FakeProjectionData::default();
    let message_id = MessageId::new();
    let sub = create_subscriber(&data);

    sub.handle(&DomainEvent::MessageCreated {
        message_id: message_id.clone(),
        app_id: ApplicationId::new(),
        event_type_id: EventTypeId::new(),
        attempts_created: attempts,
    })
    .await;

    world.message_id = Some(message_id);
    world.data = Some(data);
}

#[when("an AttemptSucceeded event is processed")]
async fn when_attempt_succeeded(world: &mut ProjectionWorld) {
    let data = world.data.as_ref().unwrap();
    let sub = create_subscriber(data);

    sub.handle(&DomainEvent::AttemptSucceeded {
        attempt_id: AttemptId::new(),
        message_id: world.message_id.clone().unwrap(),
        endpoint_id: world.endpoint_id.clone().unwrap(),
        response_code: 200,
        duration_ms: 50,
    })
    .await;
}

#[when("an AttemptFailed event with will_retry false is processed")]
async fn when_attempt_failed_final(world: &mut ProjectionWorld) {
    let data = world.data.as_ref().unwrap();
    let sub = create_subscriber(data);

    sub.handle(&DomainEvent::AttemptFailed {
        attempt_id: AttemptId::new(),
        message_id: world.message_id.clone().unwrap(),
        endpoint_id: world.endpoint_id.clone().unwrap(),
        response_code: Some(500),
        duration_ms: 10,
        will_retry: false,
    })
    .await;
}

#[when("an AttemptFailed event with will_retry true is processed")]
async fn when_attempt_failed_retry(world: &mut ProjectionWorld) {
    let data = world.data.as_ref().unwrap();
    let sub = create_subscriber(data);

    sub.handle(&DomainEvent::AttemptFailed {
        attempt_id: AttemptId::new(),
        message_id: world.message_id.clone().unwrap(),
        endpoint_id: world.endpoint_id.clone().unwrap(),
        response_code: Some(500),
        duration_ms: 10,
        will_retry: true,
    })
    .await;
}

#[when("an AttemptSucceeded event is processed for that endpoint")]
async fn when_attempt_succeeded_for_endpoint(world: &mut ProjectionWorld) {
    let data = world.data.as_ref().unwrap();
    let sub = create_subscriber(data);

    sub.handle(&DomainEvent::AttemptSucceeded {
        attempt_id: AttemptId::new(),
        message_id: world.message_id.clone().unwrap(),
        endpoint_id: world.endpoint_id.clone().unwrap(),
        response_code: 200,
        duration_ms: 50,
    })
    .await;
}

#[when("a DeadLettered event is processed")]
async fn when_dead_lettered(world: &mut ProjectionWorld) {
    let data = world.data.as_ref().unwrap();
    let sub = create_subscriber(data);

    sub.handle(&DomainEvent::DeadLettered {
        message_id: world.message_id.clone().unwrap(),
        endpoint_id: world.endpoint_id.clone().unwrap(),
        app_id: ApplicationId::new(),
    })
    .await;
}

// --- Then ---

#[then(regex = r"the message delivery status should show (\d+) attempts created")]
async fn then_attempts_created(world: &mut ProjectionWorld, expected: i32) {
    let msgs = world.data.as_ref().unwrap().messages.lock().unwrap();
    let status = msgs.get(world.message_id.as_ref().unwrap()).unwrap();
    assert_eq!(status.attempts_created, expected);
}

#[then("all delivery counters should be zero")]
async fn then_counters_zero(world: &mut ProjectionWorld) {
    let msgs = world.data.as_ref().unwrap().messages.lock().unwrap();
    let status = msgs.get(world.message_id.as_ref().unwrap()).unwrap();
    assert_eq!(status.succeeded, 0);
    assert_eq!(status.failed, 0);
    assert_eq!(status.dead_lettered, 0);
}

#[then(regex = r"the endpoint summary should show (\d+) success and (\d+) consecutive failures")]
async fn then_endpoint_summary(world: &mut ProjectionWorld, success: i64, consecutive: i64) {
    let eps = world.data.as_ref().unwrap().endpoints.lock().unwrap();
    let summary = eps.get(world.endpoint_id.as_ref().unwrap()).unwrap();
    assert_eq!(summary.total_success, success);
    assert_eq!(summary.consecutive_failures, consecutive);
}

#[then(regex = r"the endpoint summary should show (\d+) failure and (\d+) consecutive failure")]
async fn then_endpoint_failure(world: &mut ProjectionWorld, failure: i64, consecutive: i64) {
    let eps = world.data.as_ref().unwrap().endpoints.lock().unwrap();
    let summary = eps.get(world.endpoint_id.as_ref().unwrap()).unwrap();
    assert_eq!(summary.total_failure, failure);
    assert_eq!(summary.consecutive_failures, consecutive);
}

#[then(regex = r"the endpoint summary should show (\d+) consecutive failures")]
async fn then_consecutive(world: &mut ProjectionWorld, expected: i64) {
    let eps = world.data.as_ref().unwrap().endpoints.lock().unwrap();
    let summary = eps.get(world.endpoint_id.as_ref().unwrap()).unwrap();
    assert_eq!(summary.consecutive_failures, expected);
}

#[then(regex = r"the message delivery status should show (\d+) succeeded")]
async fn then_message_succeeded(world: &mut ProjectionWorld, expected: i32) {
    let msgs = world.data.as_ref().unwrap().messages.lock().unwrap();
    let status = msgs.get(world.message_id.as_ref().unwrap()).unwrap();
    assert_eq!(status.succeeded, expected);
}

#[then(regex = r"the message delivery status should show (\d+) failed")]
async fn then_message_failed(world: &mut ProjectionWorld, expected: i32) {
    let msgs = world.data.as_ref().unwrap().messages.lock().unwrap();
    let status = msgs.get(world.message_id.as_ref().unwrap()).unwrap();
    assert_eq!(status.failed, expected);
}

#[then(regex = r"the message delivery status should show (\d+) dead lettered")]
async fn then_message_dead_lettered(world: &mut ProjectionWorld, expected: i32) {
    let msgs = world.data.as_ref().unwrap().messages.lock().unwrap();
    let status = msgs.get(world.message_id.as_ref().unwrap()).unwrap();
    assert_eq!(status.dead_lettered, expected);
}

#[tokio::main]
async fn main() {
    ProjectionWorld::cucumber()
        .with_default_cli()
        .run("tests/features/delivery_projection.feature")
        .await;
}
