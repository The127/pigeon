use chrono::{DateTime, Utc};
use pigeon_domain::oidc_config::OidcConfig;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOidcConfigRequest {
    pub issuer_url: String,
    pub audience: String,
    pub jwks_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OidcConfigResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub issuer_url: String,
    pub audience: String,
    pub jwks_url: String,
    pub created_at: DateTime<Utc>,
    pub version: u64,
}

impl From<OidcConfig> for OidcConfigResponse {
    fn from(config: OidcConfig) -> Self {
        Self {
            id: *config.id().as_uuid(),
            org_id: *config.org_id().as_uuid(),
            issuer_url: config.issuer_url().to_string(),
            audience: config.audience().to_string(),
            jwks_url: config.jwks_url().to_string(),
            created_at: *config.created_at(),
            version: config.version().as_u64(),
        }
    }
}
