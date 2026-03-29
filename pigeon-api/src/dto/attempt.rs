use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_domain::attempt::Attempt;

#[derive(Debug, Serialize, ToSchema)]
pub struct AttemptResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub endpoint_id: Uuid,
    pub status: String,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    pub attempted_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub attempt_number: u32,
    pub duration_ms: Option<i64>,
}

impl From<Attempt> for AttemptResponse {
    fn from(att: Attempt) -> Self {
        Self {
            id: *att.id().as_uuid(),
            message_id: *att.message_id().as_uuid(),
            endpoint_id: *att.endpoint_id().as_uuid(),
            status: format!("{:?}", att.status()).to_lowercase(),
            response_code: att.response_code(),
            response_body: att.response_body().map(|s| s.to_string()),
            attempted_at: att.attempted_at().copied(),
            next_attempt_at: att.next_attempt_at().copied(),
            attempt_number: att.attempt_number(),
            duration_ms: att.duration_ms(),
        }
    }
}
