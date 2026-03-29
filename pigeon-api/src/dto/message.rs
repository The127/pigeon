use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::commands::send_message::SendMessageResult;
use pigeon_application::ports::message_status::MessageWithStatus;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    pub event_type_id: Uuid,
    pub payload: serde_json::Value,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SendMessageResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub event_type_id: Uuid,
    pub payload: serde_json::Value,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
    pub attempts_created: usize,
    pub was_duplicate: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub event_type_id: Uuid,
    pub payload: serde_json::Value,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
    pub attempts_created: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub dead_lettered: u32,
}

impl From<MessageWithStatus> for MessageResponse {
    fn from(mws: MessageWithStatus) -> Self {
        let msg = mws.message;
        Self {
            id: *msg.id().as_uuid(),
            app_id: *msg.app_id().as_uuid(),
            event_type_id: *msg.event_type_id().as_uuid(),
            payload: msg.payload().clone(),
            idempotency_key: msg.idempotency_key().as_str().to_string(),
            created_at: *msg.created_at(),
            attempts_created: mws.attempts_created,
            succeeded: mws.succeeded,
            failed: mws.failed,
            dead_lettered: mws.dead_lettered,
        }
    }
}

impl From<SendMessageResult> for SendMessageResponse {
    fn from(result: SendMessageResult) -> Self {
        let msg = &result.message;
        Self {
            id: *msg.id().as_uuid(),
            app_id: *msg.app_id().as_uuid(),
            event_type_id: *msg.event_type_id().as_uuid(),
            payload: msg.payload().clone(),
            idempotency_key: msg.idempotency_key().as_str().to_string(),
            created_at: *msg.created_at(),
            attempts_created: result.attempts_created,
            was_duplicate: result.was_duplicate,
        }
    }
}
