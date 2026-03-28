use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::AttemptStore;
use pigeon_domain::attempt::{Attempt, AttemptId, AttemptState, AttemptStatus};
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::message::MessageId;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgAttemptStore {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgAttemptStore {
    pub(crate) fn new(pool: PgPool, tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { pool, tracker }
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
impl AttemptStore for PgAttemptStore {
    async fn insert(&mut self, attempt: &Attempt) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertAttempt(attempt.clone()));
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &AttemptId,
        org_id: &OrganizationId,
    ) -> Result<Option<Attempt>, ApplicationError> {
        let row = sqlx::query_as::<_, AttemptRow>(
            "SELECT a.id, a.message_id, a.endpoint_id, a.status, \
             a.response_code, a.response_body, a.attempted_at, \
             a.next_attempt_at, a.attempt_number, a.duration_ms, \
             a.xmin::text::bigint AS version \
             FROM attempts a \
             JOIN messages m ON m.id = a.message_id \
             JOIN applications app ON app.id = m.app_id \
             WHERE a.id = $1 AND app.org_id = $2",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_attempt()))
    }

    async fn save(&mut self, attempt: &Attempt) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::SaveAttempt(attempt.clone()));
        Ok(())
    }
}
