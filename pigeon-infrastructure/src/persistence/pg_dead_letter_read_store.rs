use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::DeadLetterReadStore;
use pigeon_domain::endpoint::EndpointId;

pub struct PgDeadLetterReadStore {
    pool: PgPool,
}

impl PgDeadLetterReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeadLetterReadStore for PgDeadLetterReadStore {
    async fn consecutive_failure_count(
        &self,
        endpoint_id: &EndpointId,
    ) -> Result<u64, ApplicationError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM dead_letters \
             WHERE endpoint_id = $1 \
               AND dead_lettered_at > COALESCE( \
                   (SELECT MAX(attempted_at) FROM attempts \
                    WHERE endpoint_id = $1 AND status = 'succeeded'), \
                   '1970-01-01'::timestamptz \
               )",
        )
        .bind(endpoint_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }
}
