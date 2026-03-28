use async_trait::async_trait;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use pigeon_application::commands::create_oidc_config::CreateOidcConfig;
use pigeon_application::commands::delete_oidc_config::DeleteOidcConfig;
use pigeon_application::commands::replay_dead_letter::ReplayDeadLetter;
use pigeon_application::commands::retry_attempt::RetryAttempt;
use pigeon_domain::attempt::Attempt;
use pigeon_domain::dead_letter::DeadLetter;
use pigeon_application::error::ApplicationError;
use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
use pigeon_application::ports::stores::{ApplicationReadStore, OidcConfigReadStore};
use pigeon_application::queries::get_oidc_config_by_id::GetOidcConfigById;
use pigeon_application::queries::list_oidc_configs_by_org::ListOidcConfigsByOrg;
use pigeon_application::queries::PaginatedResult;
use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId};
use pigeon_domain::organization::OrganizationId;
use uuid::Uuid;

use crate::auth::{AuthContext, JwksProvider};

// --- Stub OIDC config handlers ---

pub(crate) struct StubCreateOidcConfigHandler;
#[async_trait]
impl CommandHandler<CreateOidcConfig> for StubCreateOidcConfigHandler {
    async fn handle(&self, _c: CreateOidcConfig) -> Result<OidcConfig, ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}

pub(crate) struct StubDeleteOidcConfigHandler;
#[async_trait]
impl CommandHandler<DeleteOidcConfig> for StubDeleteOidcConfigHandler {
    async fn handle(&self, _c: DeleteOidcConfig) -> Result<(), ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}

pub(crate) struct StubGetOidcConfigHandler;
#[async_trait]
impl QueryHandler<GetOidcConfigById> for StubGetOidcConfigHandler {
    async fn handle(
        &self,
        _q: GetOidcConfigById,
    ) -> Result<Option<OidcConfig>, ApplicationError> {
        Ok(None)
    }
}

pub(crate) struct StubListOidcConfigsHandler;
#[async_trait]
impl QueryHandler<ListOidcConfigsByOrg> for StubListOidcConfigsHandler {
    async fn handle(
        &self,
        _q: ListOidcConfigsByOrg,
    ) -> Result<PaginatedResult<OidcConfig>, ApplicationError> {
        Ok(PaginatedResult {
            items: vec![],
            total: 0,
            offset: 0,
            limit: 20,
        })
    }
}

// --- Stub OidcConfigReadStore ---

pub(crate) struct StubOidcConfigReadStore;
#[async_trait]
impl OidcConfigReadStore for StubOidcConfigReadStore {
    async fn find_by_issuer_and_audience(
        &self,
        _issuer_url: &str,
        _audience: &str,
    ) -> Result<Option<OidcConfig>, ApplicationError> {
        Ok(None)
    }

    async fn find_by_id(
        &self,
        _id: &OidcConfigId,
    ) -> Result<Option<OidcConfig>, ApplicationError> {
        Ok(None)
    }

    async fn list_by_org(
        &self,
        _org_id: &OrganizationId,
        _offset: u64,
        _limit: u64,
    ) -> Result<Vec<OidcConfig>, ApplicationError> {
        Ok(vec![])
    }

    async fn count_by_org(
        &self,
        _org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        Ok(0)
    }
}

// --- Stub JwksProvider ---

pub(crate) struct StubJwksProvider;
#[async_trait]
impl JwksProvider for StubJwksProvider {
    async fn get_jwks(
        &self,
        _jwks_url: &str,
    ) -> Result<jsonwebtoken::jwk::JwkSet, String> {
        Ok(jsonwebtoken::jwk::JwkSet { keys: vec![] })
    }

    async fn refresh_jwks(
        &self,
        _jwks_url: &str,
    ) -> Result<jsonwebtoken::jwk::JwkSet, String> {
        Ok(jsonwebtoken::jwk::JwkSet { keys: vec![] })
    }
}

// --- Stub ApplicationReadStore (returns None) ---

pub(crate) struct StubApplicationReadStore;
#[async_trait]
impl ApplicationReadStore for StubApplicationReadStore {
    async fn find_by_id(
        &self,
        _id: &ApplicationId,
    ) -> Result<Option<Application>, ApplicationError> {
        Ok(None)
    }
    async fn list_by_org(
        &self,
        _org_id: &OrganizationId,
        _offset: u64,
        _limit: u64,
    ) -> Result<Vec<Application>, ApplicationError> {
        Ok(vec![])
    }
    async fn count_by_org(
        &self,
        _org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        Ok(0)
    }
}

// --- Fake ApplicationReadStore (returns a configurable application) ---

pub(crate) struct FakeApplicationReadStore {
    pub(crate) app: Option<Application>,
}

#[async_trait]
impl ApplicationReadStore for FakeApplicationReadStore {
    async fn find_by_id(
        &self,
        _id: &ApplicationId,
    ) -> Result<Option<Application>, ApplicationError> {
        Ok(self.app.clone())
    }
    async fn list_by_org(
        &self,
        _org_id: &OrganizationId,
        _offset: u64,
        _limit: u64,
    ) -> Result<Vec<Application>, ApplicationError> {
        Ok(self.app.clone().into_iter().collect())
    }
    async fn count_by_org(
        &self,
        _org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        Ok(self.app.as_ref().map(|_| 1).unwrap_or(0))
    }
}

/// Test middleware that injects an AuthContext from the x-org-id header.
/// Test-only header; production uses JWT auth.
pub(crate) async fn test_auth_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(org_id_header) = request.headers().get("x-org-id") {
        if let Ok(org_id_str) = org_id_header.to_str() {
            if let Ok(uuid) = Uuid::parse_str(org_id_str) {
                let auth_context = AuthContext {
                    org_id: OrganizationId::from_uuid(uuid),
                    user_id: "test-user".to_string(),
                };
                request.extensions_mut().insert(auth_context);
            }
        }
    }
    next.run(request).await
}

pub(crate) struct StubReplayDeadLetterHandler;
#[async_trait]
impl CommandHandler<ReplayDeadLetter> for StubReplayDeadLetterHandler {
    async fn handle(&self, _c: ReplayDeadLetter) -> Result<DeadLetter, ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}

pub(crate) struct StubRetryAttemptHandler;
#[async_trait]
impl CommandHandler<RetryAttempt> for StubRetryAttemptHandler {
    async fn handle(&self, _c: RetryAttempt) -> Result<Attempt, ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}
