use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::DeadLetterStore;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::dead_letter::{DeadLetter, DeadLetterId, DeadLetterState};
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::message::MessageId;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgDeadLetterStore {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgDeadLetterStore {
    pub(crate) fn new(pool: PgPool, tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { pool, tracker }
    }
}

#[derive(sqlx::FromRow)]
struct DeadLetterRow {
    id: uuid::Uuid,
    message_id: uuid::Uuid,
    endpoint_id: uuid::Uuid,
    app_id: uuid::Uuid,
    last_response_code: Option<i16>,
    last_response_body: Option<String>,
    dead_lettered_at: chrono::DateTime<chrono::Utc>,
    replayed_at: Option<chrono::DateTime<chrono::Utc>>,
    version: i64,
}

impl DeadLetterRow {
    fn into_dead_letter(self) -> DeadLetter {
        DeadLetter::reconstitute(DeadLetterState {
            id: DeadLetterId::from_uuid(self.id),
            message_id: MessageId::from_uuid(self.message_id),
            endpoint_id: EndpointId::from_uuid(self.endpoint_id),
            app_id: ApplicationId::from_uuid(self.app_id),
            last_response_code: self.last_response_code.map(|c| c as u16),
            last_response_body: self.last_response_body,
            dead_lettered_at: self.dead_lettered_at,
            replayed_at: self.replayed_at,
            version: Version::new(self.version as u64),
        })
    }
}

#[async_trait]
impl DeadLetterStore for PgDeadLetterStore {
    async fn insert(&mut self, dead_letter: &DeadLetter) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertDeadLetter(dead_letter.clone()));
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &DeadLetterId,
        org_id: &OrganizationId,
    ) -> Result<Option<DeadLetter>, ApplicationError> {
        let row = sqlx::query_as::<_, DeadLetterRow>(
            "SELECT dl.id, dl.message_id, dl.endpoint_id, dl.app_id, \
             dl.last_response_code, dl.last_response_body, \
             dl.dead_lettered_at, dl.replayed_at, \
             dl.xmin::text::bigint AS version \
             FROM dead_letters dl \
             JOIN applications a ON a.id = dl.app_id \
             WHERE dl.id = $1 AND a.org_id = $2",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_dead_letter()))
    }

    async fn save(&mut self, dead_letter: &DeadLetter) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::SaveDeadLetter(dead_letter.clone()));
        Ok(())
    }
}
