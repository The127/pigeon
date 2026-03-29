use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::projection_store::ProjectionStore;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::message::MessageId;

pub struct PgProjectionStore {
    pool: PgPool,
}

impl PgProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectionStore for PgProjectionStore {
    async fn record_endpoint_success(
        &self,
        endpoint_id: &EndpointId,
        delivered_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO endpoint_delivery_summary \
             (endpoint_id, last_delivery_at, last_status, total_success, consecutive_failures) \
             VALUES ($1, $2, 'succeeded', 1, 0) \
             ON CONFLICT (endpoint_id) DO UPDATE SET \
                 last_delivery_at = $2, \
                 last_status = 'succeeded', \
                 total_success = endpoint_delivery_summary.total_success + 1, \
                 consecutive_failures = 0",
        )
        .bind(endpoint_id.as_uuid())
        .bind(delivered_at)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn record_endpoint_failure(
        &self,
        endpoint_id: &EndpointId,
        delivered_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO endpoint_delivery_summary \
             (endpoint_id, last_delivery_at, last_status, total_failure, consecutive_failures) \
             VALUES ($1, $2, 'failed', 1, 1) \
             ON CONFLICT (endpoint_id) DO UPDATE SET \
                 last_delivery_at = $2, \
                 last_status = 'failed', \
                 total_failure = endpoint_delivery_summary.total_failure + 1, \
                 consecutive_failures = endpoint_delivery_summary.consecutive_failures + 1",
        )
        .bind(endpoint_id.as_uuid())
        .bind(delivered_at)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn init_message_status(
        &self,
        message_id: &MessageId,
        attempts_created: u32,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO message_delivery_status \
             (message_id, attempts_created) \
             VALUES ($1, $2) \
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(message_id.as_uuid())
        .bind(attempts_created as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn add_message_attempts(
        &self,
        message_id: &MessageId,
        count: u32,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "UPDATE message_delivery_status \
             SET attempts_created = attempts_created + $2 \
             WHERE message_id = $1",
        )
        .bind(message_id.as_uuid())
        .bind(count as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn increment_message_succeeded(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "UPDATE message_delivery_status \
             SET succeeded = succeeded + 1 \
             WHERE message_id = $1",
        )
        .bind(message_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn increment_message_failed(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "UPDATE message_delivery_status \
             SET failed = failed + 1 \
             WHERE message_id = $1",
        )
        .bind(message_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn increment_message_dead_lettered(
        &self,
        message_id: &MessageId,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "UPDATE message_delivery_status \
             SET dead_lettered = dead_lettered + 1 \
             WHERE message_id = $1",
        )
        .bind(message_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }
}
