use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::delivery::{DeliveryQueue, DeliveryTask};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::attempt::AttemptId;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::event::DomainEvent;
use pigeon_domain::message::MessageId;

pub struct PgDeliveryQueue {
    pool: PgPool,
}

impl PgDeliveryQueue {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn insert_outbox_event(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        event: &DomainEvent,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO event_outbox (id, event_type, payload) VALUES ($1, $2, $3)",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(event.event_type())
        .bind(event.to_json())
        .execute(&mut **tx)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct DeliveryTaskRow {
    attempt_id: uuid::Uuid,
    endpoint_url: String,
    signing_secret: String,
    payload: serde_json::Value,
    attempt_number: i32,
    endpoint_id: uuid::Uuid,
    message_id: uuid::Uuid,
    app_id: uuid::Uuid,
}

#[async_trait]
impl DeliveryQueue for PgDeliveryQueue {
    async fn dequeue(&self, batch_size: u32) -> Result<Vec<DeliveryTask>, ApplicationError> {
        let rows = sqlx::query_as::<_, DeliveryTaskRow>(
            "UPDATE attempts \
             SET status = 'in_flight', \
                 attempt_number = attempt_number + 1 \
             WHERE id IN ( \
                 SELECT a.id FROM attempts a \
                 WHERE a.status = 'pending' \
                   AND a.next_attempt_at <= now() \
                 ORDER BY a.next_attempt_at \
                 LIMIT $1 \
                 FOR UPDATE SKIP LOCKED \
             ) \
             RETURNING \
                 attempts.id AS attempt_id, \
                 attempts.attempt_number, \
                 attempts.endpoint_id, \
                 attempts.message_id, \
                 (SELECT e.url FROM endpoints e WHERE e.id = attempts.endpoint_id) AS endpoint_url, \
                 (SELECT e.signing_secret FROM endpoints e WHERE e.id = attempts.endpoint_id) AS signing_secret, \
                 (SELECT m.payload FROM messages m WHERE m.id = attempts.message_id) AS payload, \
                 (SELECT m.app_id FROM messages m WHERE m.id = attempts.message_id) AS app_id",
        )
        .bind(batch_size as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| DeliveryTask {
                attempt_id: AttemptId::from_uuid(r.attempt_id),
                endpoint_url: r.endpoint_url,
                signing_secret: r.signing_secret,
                payload: r.payload,
                attempt_number: r.attempt_number as u32,
                endpoint_id: EndpointId::from_uuid(r.endpoint_id),
                message_id: MessageId::from_uuid(r.message_id),
                app_id: ApplicationId::from_uuid(r.app_id),
            })
            .collect())
    }

    async fn record_success(
        &self,
        attempt_id: &AttemptId,
        message_id: &MessageId,
        endpoint_id: &EndpointId,
        response_code: u16,
        response_body: String,
        duration_ms: i64,
    ) -> Result<(), ApplicationError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE attempts \
             SET status = 'succeeded', \
                 response_code = $2, \
                 response_body = $3, \
                 duration_ms = $4, \
                 attempted_at = now(), \
                 next_attempt_at = NULL \
             WHERE id = $1",
        )
        .bind(attempt_id.as_uuid())
        .bind(response_code as i16)
        .bind(&response_body)
        .bind(duration_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let event = DomainEvent::AttemptSucceeded {
            attempt_id: attempt_id.clone(),
            message_id: message_id.clone(),
            endpoint_id: endpoint_id.clone(),
            response_code,
            duration_ms,
        };
        Self::insert_outbox_event(&mut tx, &event).await?;

        tx.commit().await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn record_failure(
        &self,
        attempt_id: &AttemptId,
        message_id: &MessageId,
        endpoint_id: &EndpointId,
        response_code: Option<u16>,
        response_body: Option<String>,
        duration_ms: i64,
        next_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), ApplicationError> {
        let will_retry = next_attempt_at.is_some();
        let status = if will_retry { "pending" } else { "failed" };

        let mut tx = self.pool.begin().await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE attempts \
             SET status = $2, \
                 response_code = $3, \
                 response_body = $4, \
                 duration_ms = $5, \
                 attempted_at = now(), \
                 next_attempt_at = $6 \
             WHERE id = $1",
        )
        .bind(attempt_id.as_uuid())
        .bind(status)
        .bind(response_code.map(|c| c as i16))
        .bind(&response_body)
        .bind(duration_ms)
        .bind(next_attempt_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let event = DomainEvent::AttemptFailed {
            attempt_id: attempt_id.clone(),
            message_id: message_id.clone(),
            endpoint_id: endpoint_id.clone(),
            response_code,
            duration_ms,
            will_retry,
        };
        Self::insert_outbox_event(&mut tx, &event).await?;

        tx.commit().await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn insert_dead_letter(
        &self,
        endpoint_id: &EndpointId,
        message_id: &MessageId,
        app_id: &ApplicationId,
        last_response_code: Option<u16>,
        last_response_body: Option<String>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO dead_letters \
             (id, message_id, endpoint_id, app_id, last_response_code, last_response_body, dead_lettered_at) \
             VALUES ($1, $2, $3, $4, $5, $6, now())",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(message_id.as_uuid())
        .bind(endpoint_id.as_uuid())
        .bind(app_id.as_uuid())
        .bind(last_response_code.map(|c| c as i16))
        .bind(last_response_body)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn expire_idempotency_keys(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, ApplicationError> {
        let result = sqlx::query("DELETE FROM messages WHERE idempotency_expires_at <= $1")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
