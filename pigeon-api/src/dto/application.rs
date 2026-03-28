use chrono::{DateTime, Utc};
use pigeon_domain::application::Application;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApplicationRequest {
    pub name: String,
    pub uid: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateApplicationRequest {
    pub name: String,
    pub version: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub uid: String,
    pub created_at: DateTime<Utc>,
    pub version: u64,
}

impl From<Application> for ApplicationResponse {
    fn from(app: Application) -> Self {
        Self {
            id: *app.id().as_uuid(),
            org_id: *app.org_id().as_uuid(),
            name: app.name().to_string(),
            uid: app.uid().to_string(),
            created_at: *app.created_at(),
            version: app.version().as_u64(),
        }
    }
}
