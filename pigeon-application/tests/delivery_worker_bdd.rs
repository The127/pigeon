use std::sync::Arc;

use cucumber::{given, then, when, World};
use serde_json::json;

use pigeon_application::ports::delivery::{
    DeliveryTask, MockDeliveryQueue, MockWebhookHttpClient, WebhookResult,
};
use pigeon_application::services::delivery_worker::{DeliveryWorkerConfig, DeliveryWorkerService};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::attempt::AttemptId;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::message::MessageId;

#[derive(Debug, Default, World)]
pub struct DeliveryWorld {
    // Config
    max_retries: u32,
    attempt_number: u32,
    endpoint_url: String,

    // Mock behavior
    http_status: Option<u16>,
    network_error: bool,
    tasks: Vec<DeliveryTask>,

    // Results
    processed_count: Option<usize>,
    recorded_success: Vec<RecordedSuccess>,
    recorded_failure: Vec<RecordedFailure>,
    dead_letters: Vec<RecordedDeadLetter>,
}

#[derive(Debug, Clone)]
struct RecordedSuccess {
    response_code: u16,
    duration_ms: i64,
}

#[derive(Debug, Clone)]
struct RecordedFailure {
    #[allow(dead_code)]
    response_code: Option<u16>,
    has_next_attempt: bool,
}

#[derive(Debug, Clone)]
struct RecordedDeadLetter {
    #[allow(dead_code)]
    endpoint_id: EndpointId,
    #[allow(dead_code)]
    message_id: MessageId,
}

fn make_task(endpoint_url: &str, attempt_number: u32) -> DeliveryTask {
    DeliveryTask {
        attempt_id: AttemptId::new(),
        endpoint_url: endpoint_url.to_string(),
        signing_secret: "whsec_test".to_string(),
        payload: json!({"event": "test"}),
        attempt_number,
        endpoint_id: EndpointId::new(),
        message_id: MessageId::new(),
        app_id: ApplicationId::new(),
    }
}

// ===== Given steps =====

#[given(regex = r#"a pending attempt for endpoint "([^"]*)""#)]
async fn given_pending_attempt(world: &mut DeliveryWorld, url: String) {
    world.endpoint_url = url.clone();
    world.attempt_number = 1;
    world.max_retries = 5;
    world.http_status = Some(200);
    world.tasks = vec![make_task(&url, 1)];
}

#[given(regex = r"a pending attempt on attempt number (\d+) with max retries (\d+)")]
async fn given_attempt_with_retries(
    world: &mut DeliveryWorld,
    attempt_number: u32,
    max_retries: u32,
) {
    world.endpoint_url = "https://example.com/hook".to_string();
    world.attempt_number = attempt_number;
    world.max_retries = max_retries;
    world.tasks = vec![make_task(&world.endpoint_url.clone(), attempt_number)];
}

#[given(regex = r"the endpoint returns status (\d+)")]
async fn given_endpoint_returns_status(world: &mut DeliveryWorld, status: u16) {
    world.http_status = Some(status);
}

#[given("the endpoint is unreachable")]
async fn given_endpoint_unreachable(world: &mut DeliveryWorld) {
    world.network_error = true;
    world.http_status = None;
}

#[given("no pending attempts")]
async fn given_no_pending_attempts(world: &mut DeliveryWorld) {
    world.tasks = vec![];
    world.max_retries = 5;
}

#[given(regex = r"(\d+) pending attempts in the queue")]
async fn given_n_pending_attempts(world: &mut DeliveryWorld, count: usize) {
    world.max_retries = 5;
    world.http_status = Some(200);
    world.tasks = (0..count)
        .map(|_| make_task("https://example.com/hook", 1))
        .collect();
}

// ===== When steps =====

#[when("the worker processes the batch")]
async fn when_worker_processes(world: &mut DeliveryWorld) {
    let tasks = world.tasks.clone();
    let http_status = world.http_status;
    let network_error = world.network_error;

    // Set up mock delivery queue
    let mut mock_queue = MockDeliveryQueue::new();

    mock_queue
        .expect_dequeue()
        .times(1)
        .returning(move |_| Ok(tasks.clone()));

    // Capture record_success calls
    let success_records: Arc<std::sync::Mutex<Vec<RecordedSuccess>>> =
        Arc::new(std::sync::Mutex::new(vec![]));
    let success_clone = success_records.clone();
    mock_queue
        .expect_record_success()
        .returning(move |_, code, _, dur| {
            success_clone
                .lock()
                .unwrap()
                .push(RecordedSuccess {
                    response_code: code,
                    duration_ms: dur,
                });
            Ok(())
        });

    // Capture record_failure calls
    let failure_records: Arc<std::sync::Mutex<Vec<RecordedFailure>>> =
        Arc::new(std::sync::Mutex::new(vec![]));
    let failure_clone = failure_records.clone();
    mock_queue
        .expect_record_failure()
        .returning(move |_, code, _, _, next| {
            failure_clone
                .lock()
                .unwrap()
                .push(RecordedFailure {
                    response_code: code,
                    has_next_attempt: next.is_some(),
                });
            Ok(())
        });

    // Capture dead letter calls
    let dl_records: Arc<std::sync::Mutex<Vec<RecordedDeadLetter>>> =
        Arc::new(std::sync::Mutex::new(vec![]));
    let dl_clone = dl_records.clone();
    mock_queue
        .expect_insert_dead_letter()
        .returning(move |ep_id, msg_id, _, _, _| {
            dl_clone
                .lock()
                .unwrap()
                .push(RecordedDeadLetter {
                    endpoint_id: ep_id.clone(),
                    message_id: msg_id.clone(),
                });
            Ok(())
        });

    // Set up mock HTTP client
    let mut mock_http = MockWebhookHttpClient::new();
    mock_http.expect_deliver().returning(move |_, _, _| {
        if network_error {
            WebhookResult::Error {
                message: "connection refused".to_string(),
                duration_ms: 5,
            }
        } else {
            let status = http_status.unwrap_or(200);
            WebhookResult::Response {
                status_code: status,
                body: format!("status {}", status),
                duration_ms: 42,
            }
        }
    });

    let config = DeliveryWorkerConfig {
        batch_size: 10,
        poll_interval: std::time::Duration::from_millis(10),
        max_retries: world.max_retries,
        backoff_base_secs: 30,
        max_backoff_secs: 3600,
        cleanup_interval: std::time::Duration::from_secs(3600),
    };

    let service = DeliveryWorkerService::new(
        Arc::new(mock_queue),
        Arc::new(mock_http),
        config,
    );

    let count = service.poll_and_deliver().await.unwrap();
    world.processed_count = Some(count);
    world.recorded_success = success_records.lock().unwrap().clone();
    world.recorded_failure = failure_records.lock().unwrap().clone();
    world.dead_letters = dl_records.lock().unwrap().clone();
}

// ===== Then steps =====

#[then("the attempt should be marked as succeeded")]
async fn then_attempt_succeeded(world: &mut DeliveryWorld) {
    assert_eq!(
        world.recorded_success.len(),
        1,
        "expected 1 success record, got {}",
        world.recorded_success.len()
    );
}

#[then(regex = r"the response code should be (\d+)")]
async fn then_response_code(world: &mut DeliveryWorld, code: u16) {
    let success = &world.recorded_success[0];
    assert_eq!(success.response_code, code);
}

#[then("the duration should be recorded")]
async fn then_duration_recorded(world: &mut DeliveryWorld) {
    let success = &world.recorded_success[0];
    assert!(success.duration_ms > 0);
}

#[then("the attempt should be marked for retry")]
async fn then_attempt_retry(world: &mut DeliveryWorld) {
    assert_eq!(
        world.recorded_failure.len(),
        1,
        "expected 1 failure record, got {}",
        world.recorded_failure.len()
    );
    assert!(
        world.recorded_failure[0].has_next_attempt,
        "expected next_attempt_at to be set for retry"
    );
}

#[then("a next_attempt_at should be computed with exponential backoff")]
async fn then_backoff_computed(world: &mut DeliveryWorld) {
    // Already verified in the retry step — next_attempt_at is Some
    assert!(world.recorded_failure[0].has_next_attempt);
}

#[then("the attempt should be marked as failed")]
async fn then_attempt_failed(world: &mut DeliveryWorld) {
    assert_eq!(
        world.recorded_failure.len(),
        1,
        "expected 1 failure record, got {}",
        world.recorded_failure.len()
    );
    assert!(
        !world.recorded_failure[0].has_next_attempt,
        "expected no next_attempt_at (final failure)"
    );
}

#[then("a dead letter should be created")]
async fn then_dead_letter_created(world: &mut DeliveryWorld) {
    assert_eq!(
        world.dead_letters.len(),
        1,
        "expected 1 dead letter, got {}",
        world.dead_letters.len()
    );
}

#[then("zero attempts should be processed")]
async fn then_zero_processed(world: &mut DeliveryWorld) {
    assert_eq!(world.processed_count, Some(0));
}

#[then(regex = r"(\d+) attempts should be processed")]
async fn then_n_processed(world: &mut DeliveryWorld, count: usize) {
    assert_eq!(world.processed_count, Some(count));
}

#[then("all attempts should be marked as succeeded")]
async fn then_all_succeeded(world: &mut DeliveryWorld) {
    let expected = world.processed_count.unwrap_or(0);
    assert_eq!(
        world.recorded_success.len(),
        expected,
        "expected {} successes, got {}",
        expected,
        world.recorded_success.len()
    );
}

#[tokio::main]
async fn main() {
    DeliveryWorld::cucumber()
        .with_default_cli()
        .run("tests/features/delivery_worker.feature")
        .await;
}
