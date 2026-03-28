use chrono::{DateTime, Utc};
use pigeon_domain::endpoint::Endpoint;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEndpointRequest {
    pub url: String,
    pub signing_secret: String,
    pub event_type_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEndpointRequest {
    pub url: String,
    pub signing_secret: String,
    pub event_type_ids: Vec<Uuid>,
    pub version: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EndpointResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub url: String,
    pub enabled: bool,
    pub event_type_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub version: u64,
}

impl From<Endpoint> for EndpointResponse {
    fn from(ep: Endpoint) -> Self {
        Self {
            id: *ep.id().as_uuid(),
            app_id: *ep.app_id().as_uuid(),
            url: ep.url().to_string(),
            enabled: ep.enabled(),
            event_type_ids: ep.event_type_ids().iter().map(|id| *id.as_uuid()).collect(),
            created_at: *ep.created_at(),
            version: ep.version().as_u64(),
        }
    }
}
