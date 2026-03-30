use chrono::{DateTime, Utc};
use pigeon_domain::endpoint::{mask_signing_secret, Endpoint};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEndpointRequest {
    pub name: Option<String>,
    pub url: String,
    pub event_type_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEndpointRequest {
    pub url: String,
    pub event_type_ids: Vec<Uuid>,
    pub version: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EndpointResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub signing_secrets_masked: Vec<String>,
    pub event_type_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub version: u64,
}

impl From<Endpoint> for EndpointResponse {
    fn from(ep: Endpoint) -> Self {
        let signing_secrets_masked = ep
            .signing_secrets()
            .iter()
            .map(|s| mask_signing_secret(s))
            .collect();
        Self {
            id: *ep.id().as_uuid(),
            app_id: *ep.app_id().as_uuid(),
            name: ep.name().to_string(),
            url: ep.url().to_string(),
            enabled: ep.enabled(),
            signing_secrets_masked,
            event_type_ids: ep.event_type_ids().iter().map(|id| *id.as_uuid()).collect(),
            created_at: *ep.created_at(),
            version: ep.version().as_u64(),
        }
    }
}
