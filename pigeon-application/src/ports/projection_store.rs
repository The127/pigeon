use async_trait::async_trait;

use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::message::MessageId;

use crate::error::ApplicationError;

#[async_trait]
pub trait ProjectionStore: Send + Sync {
    /// Record a successful delivery for an endpoint.
    async fn record_endpoint_success(
        &self,
        endpoint_id: &EndpointId,
        delivered_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ApplicationError>;

    /// Record a failed delivery for an endpoint.
    async fn record_endpoint_failure(
        &self,
        endpoint_id: &EndpointId,
        delivered_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ApplicationError>;

    /// Initialize message delivery status with the number of attempts created.
    async fn init_message_status(
        &self,
        message_id: &MessageId,
        attempts_created: u32,
    ) -> Result<(), ApplicationError>;

    /// Increment the succeeded count for a message.
    async fn increment_message_succeeded(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError>;

    /// Increment the failed count for a message.
    async fn increment_message_failed(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError>;

    /// Increment the dead_lettered count for a message.
    async fn increment_message_dead_lettered(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError>;
}
