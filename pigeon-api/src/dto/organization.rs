use chrono::{DateTime, Utc};
use pigeon_domain::organization::Organization;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub slug: String,
    pub oidc_issuer_url: String,
    pub oidc_audience: String,
    pub oidc_jwks_url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub version: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub version: u64,
}

impl From<Organization> for OrganizationResponse {
    fn from(org: Organization) -> Self {
        Self {
            id: *org.id().as_uuid(),
            name: org.name().to_string(),
            slug: org.slug().to_string(),
            created_at: *org.created_at(),
            version: org.version().as_u64(),
        }
    }
}
