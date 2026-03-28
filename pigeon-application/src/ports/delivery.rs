use async_trait::async_trait;
use serde_json::Value;

use pigeon_domain::application::ApplicationId;
use pigeon_domain::attempt::AttemptId;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::message::MessageId;

use crate::error::ApplicationError;

/// A task dequeued from the delivery queue, containing everything
/// needed to deliver a webhook and record the outcome.
#[derive(Debug, Clone)]
pub struct DeliveryTask {
    pub attempt_id: AttemptId,
    pub endpoint_url: String,
    pub signing_secret: String,
    pub payload: Value,
    pub attempt_number: u32,
    pub endpoint_id: EndpointId,
    pub message_id: MessageId,
    pub app_id: ApplicationId,
}

/// Outcome of an HTTP webhook delivery attempt.
#[derive(Debug)]
pub enum WebhookResult {
    /// HTTP response received (may be success or error status).
    Response {
        status_code: u16,
        body: String,
        duration_ms: i64,
    },
    /// Network or timeout error — no HTTP response received.
    Error {
        message: String,
        duration_ms: i64,
    },
}

/// Port for dequeuing and recording delivery attempts.
///
/// Uses direct SQL transactions (not the change tracker UoW) because
/// the worker must hold locks during HTTP delivery.
#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait DeliveryQueue: Send + Sync {
    /// Atomically claim up to `batch_size` pending attempts.
    /// Sets status to `in_flight` and increments `attempt_number`.
    async fn dequeue(&self, batch_size: u32) -> Result<Vec<DeliveryTask>, ApplicationError>;

    /// Record a successful delivery.
    async fn record_success(
        &self,
        attempt_id: &AttemptId,
        response_code: u16,
        response_body: String,
        duration_ms: i64,
    ) -> Result<(), ApplicationError>;

    /// Record a failed delivery. If `next_attempt_at` is Some, sets status
    /// back to pending for retry. If None, sets status to failed (dead letter).
    async fn record_failure(
        &self,
        attempt_id: &AttemptId,
        response_code: Option<u16>,
        response_body: Option<String>,
        duration_ms: i64,
        next_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), ApplicationError>;

    /// Insert a dead letter record after retry exhaustion.
    async fn insert_dead_letter(
        &self,
        endpoint_id: &EndpointId,
        message_id: &MessageId,
        app_id: &ApplicationId,
        last_response_code: Option<u16>,
        last_response_body: Option<String>,
    ) -> Result<(), ApplicationError>;
}

/// Port for HTTP webhook delivery.
#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait WebhookHttpClient: Send + Sync {
    /// Deliver a webhook payload to the given URL, signing with HMAC-SHA256.
    async fn deliver(
        &self,
        url: &str,
        payload: &Value,
        signing_secret: &str,
    ) -> WebhookResult;
}
