use std::sync::Arc;

use cucumber::{given, then, when, World};

use pigeon_application::commands::disable_endpoint::DisableEndpointHandler;
use pigeon_application::ports::stores::MockDeadLetterReadStore;
use pigeon_application::services::auto_disable_saga::AutoDisableEndpointSaga;
use pigeon_application::services::outbox_worker::EventSubscriber;
use pigeon_application::test_support::fakes::{
    FakeUnitOfWorkFactory, OperationLog, SharedEndpointData,
};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::{Endpoint, EndpointId};
use pigeon_domain::event::DomainEvent;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::message::MessageId;

#[derive(Debug, Default, World)]
pub struct AutoDisableWorld {
    endpoint_id: Option<EndpointId>,
    app_id: Option<ApplicationId>,
    ep_data: Option<SharedEndpointData>,
    log: Option<OperationLog>,
    threshold: u64,
    consecutive_failures: u64,
    initially_disabled: bool,
}

#[given(regex = r"an endpoint with (\d+) consecutive failures and a threshold of (\d+)")]
async fn given_endpoint_with_failures(
    world: &mut AutoDisableWorld,
    failures: u64,
    threshold: u64,
) {
    let app_id = ApplicationId::new();
    let ep = Endpoint::new(
        app_id.clone(),
        None,
        "https://example.com/hook".into(),
        Some("whsec_test".into()),
        vec![EventTypeId::new()],
    )
    .unwrap();

    let ep_data = SharedEndpointData::default();
    ep_data.endpoints.lock().unwrap().push(ep.clone());

    world.endpoint_id = Some(ep.id().clone());
    world.app_id = Some(app_id);
    world.ep_data = Some(ep_data);
    world.log = Some(OperationLog::new());
    world.threshold = threshold;
    world.consecutive_failures = failures;
    world.initially_disabled = false;
}

#[given(regex = r"an already disabled endpoint with (\d+) consecutive failures and a threshold of (\d+)")]
async fn given_disabled_endpoint(
    world: &mut AutoDisableWorld,
    failures: u64,
    threshold: u64,
) {
    let app_id = ApplicationId::new();
    let mut ep = Endpoint::new(
        app_id.clone(),
        None,
        "https://example.com/hook".into(),
        Some("whsec_test".into()),
        vec![EventTypeId::new()],
    )
    .unwrap();
    ep.disable();

    let ep_data = SharedEndpointData::default();
    ep_data.endpoints.lock().unwrap().push(ep.clone());

    world.endpoint_id = Some(ep.id().clone());
    world.app_id = Some(app_id);
    world.ep_data = Some(ep_data);
    world.log = Some(OperationLog::new());
    world.threshold = threshold;
    world.consecutive_failures = failures;
    world.initially_disabled = true;
}

#[when("the DeadLettered event is processed")]
async fn when_dead_lettered(world: &mut AutoDisableWorld) {
    let endpoint_id = world.endpoint_id.clone().unwrap();
    let app_id = world.app_id.clone().unwrap();
    let failures = world.consecutive_failures;

    let mut mock_read_store = MockDeadLetterReadStore::new();
    mock_read_store
        .expect_consecutive_failure_count()
        .returning(move |_| Ok(failures));

    let log = world.log.as_ref().unwrap().clone();
    let ep_data = world.ep_data.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_endpoint_data(log, ep_data));

    let saga = AutoDisableEndpointSaga::new(
        Arc::new(mock_read_store),
        Arc::new(DisableEndpointHandler::new(factory)),
        world.threshold,
    );

    let event = DomainEvent::DeadLettered {
        message_id: MessageId::new(),
        endpoint_id,
        app_id,
    };

    saga.handle(&event).await;
}

#[then("the endpoint should be disabled")]
async fn then_disabled(world: &mut AutoDisableWorld) {
    let ep_data = world.ep_data.as_ref().unwrap();
    let eps = ep_data.endpoints.lock().unwrap();
    let ep = eps.iter().find(|e| e.id() == world.endpoint_id.as_ref().unwrap()).unwrap();
    assert!(!ep.enabled(), "expected endpoint to be disabled");
}

#[then("the endpoint should remain enabled")]
async fn then_still_enabled(world: &mut AutoDisableWorld) {
    let ep_data = world.ep_data.as_ref().unwrap();
    let eps = ep_data.endpoints.lock().unwrap();
    let ep = eps.iter().find(|e| e.id() == world.endpoint_id.as_ref().unwrap()).unwrap();
    assert!(ep.enabled(), "expected endpoint to remain enabled");
}

#[then("the endpoint should remain disabled")]
async fn then_still_disabled(world: &mut AutoDisableWorld) {
    let ep_data = world.ep_data.as_ref().unwrap();
    let eps = ep_data.endpoints.lock().unwrap();
    let ep = eps.iter().find(|e| e.id() == world.endpoint_id.as_ref().unwrap()).unwrap();
    assert!(!ep.enabled(), "expected endpoint to remain disabled");
}

#[tokio::main]
async fn main() {
    AutoDisableWorld::cucumber()
        .with_default_cli()
        .run("tests/features/auto_disable.feature")
        .await;
}
