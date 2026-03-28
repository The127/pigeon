pub mod applications;
pub mod endpoints;
pub mod event_types;
pub mod health;
pub mod messages;
pub(crate) mod oidc_configs;
pub mod organizations;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::ApplicationReadStore;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApiError;

/// Verify that the application belongs to the given organization.
/// Returns NotFound if the app doesn't exist or belongs to a different org.
pub(crate) async fn verify_app_ownership(
    read_store: &dyn ApplicationReadStore,
    app_id: &ApplicationId,
    org_id: &OrganizationId,
) -> Result<(), ApiError> {
    let app = read_store.find_by_id(app_id).await.map_err(ApiError)?;
    match app {
        Some(a) if a.org_id() == org_id => Ok(()),
        _ => Err(ApiError(ApplicationError::NotFound)),
    }
}
