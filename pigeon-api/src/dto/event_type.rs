use chrono::{DateTime, Utc};
use pigeon_domain::event_type::EventType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEventTypeRequest {
    pub name: String,
    pub schema: Option<Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEventTypeRequest {
    pub name: String,
    pub schema: Option<Value>,
    pub version: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventTypeResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub name: String,
    pub schema: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub version: u64,
}

impl From<EventType> for EventTypeResponse {
    fn from(et: EventType) -> Self {
        Self {
            id: *et.id().as_uuid(),
            app_id: *et.app_id().as_uuid(),
            name: et.name().to_string(),
            schema: et.schema().cloned(),
            created_at: *et.created_at(),
            version: et.version().as_u64(),
        }
    }
}
