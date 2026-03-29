use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_domain::dead_letter::DeadLetter;

#[derive(Debug, Serialize, ToSchema)]
pub struct DeadLetterResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub endpoint_id: Uuid,
    pub app_id: Uuid,
    pub last_response_code: Option<u16>,
    pub last_response_body: Option<String>,
    pub dead_lettered_at: DateTime<Utc>,
    pub replayed_at: Option<DateTime<Utc>>,
}

impl From<DeadLetter> for DeadLetterResponse {
    fn from(dl: DeadLetter) -> Self {
        Self {
            id: *dl.id().as_uuid(),
            message_id: *dl.message_id().as_uuid(),
            endpoint_id: *dl.endpoint_id().as_uuid(),
            app_id: *dl.app_id().as_uuid(),
            last_response_code: dl.last_response_code(),
            last_response_body: dl.last_response_body().map(|s| s.to_string()),
            dead_lettered_at: *dl.dead_lettered_at(),
            replayed_at: dl.replayed_at().copied(),
        }
    }
}
