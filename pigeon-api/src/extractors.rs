use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use pigeon_domain::organization::OrganizationId;

use crate::auth::AuthContext;

/// Extracts the organization ID from the authenticated request context.
/// The auth middleware populates this from JWT validation.
pub struct OrgId(pub OrganizationId);

impl<S: Send + Sync> FromRequestParts<S> for OrgId {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts
            .extensions
            .get::<AuthContext>()
            .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))?;
        Ok(OrgId(auth.org_id.clone()))
    }
}
