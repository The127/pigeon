use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::AttemptReadStore;
use pigeon_domain::attempt::{Attempt, AttemptId, AttemptState, AttemptStatus};
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::message::MessageId;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;

pub struct PgAttemptReadStore {
    pool: PgPool,
}

impl PgAttemptReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AttemptRow {
    id: uuid::Uuid,
    message_id: uuid::Uuid,
    endpoint_id: uuid::Uuid,
    status: String,
    response_code: Option<i16>,
    response_body: Option<String>,
    attempted_at: Option<chrono::DateTime<chrono::Utc>>,
    next_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    attempt_number: i32,
    duration_ms: Option<i64>,
    version: i64,
}

impl AttemptRow {
    fn into_attempt(self) -> Attempt {
        let status = match self.status.as_str() {
            "pending" => AttemptStatus::Pending,
            "in_flight" => AttemptStatus::InFlight,
            "succeeded" => AttemptStatus::Succeeded,
            "failed" => AttemptStatus::Failed,
            _ => AttemptStatus::Failed,
        };
        Attempt::reconstitute(AttemptState {
            id: AttemptId::from_uuid(self.id),
            message_id: MessageId::from_uuid(self.message_id),
            endpoint_id: EndpointId::from_uuid(self.endpoint_id),
            status,
            response_code: self.response_code.map(|c| c as u16),
            response_body: self.response_body,
            attempted_at: self.attempted_at,
            next_attempt_at: self.next_attempt_at,
            attempt_number: self.attempt_number as u32,
            duration_ms: self.duration_ms,
            version: Version::new(self.version as u64),
        })
    }
}

#[async_trait]
impl AttemptReadStore for PgAttemptReadStore {
    async fn list_by_message(
        &self,
        message_id: &MessageId,
        org_id: &OrganizationId,
    ) -> Result<Vec<Attempt>, ApplicationError> {
        let rows = sqlx::query_as::<_, AttemptRow>(
            "SELECT att.id, att.message_id, att.endpoint_id, att.status, \
                    att.response_code, att.response_body, att.attempted_at, \
                    att.next_attempt_at, att.attempt_number, att.duration_ms, \
                    att.xmin::text::bigint AS version \
             FROM attempts att \
             JOIN messages m ON m.id = att.message_id \
             JOIN applications a ON a.id = m.app_id \
             WHERE att.message_id = $1 AND a.org_id = $2 \
             ORDER BY att.attempt_number ASC",
        )
        .bind(message_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_attempt()).collect())
    }
}
