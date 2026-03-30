use async_trait::async_trait;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use pigeon_application::commands::create_oidc_config::CreateOidcConfig;
use pigeon_application::commands::delete_oidc_config::DeleteOidcConfig;
use pigeon_application::commands::replay_dead_letter::ReplayDeadLetter;
use pigeon_application::commands::retrigger_message::RetriggerMessage;
use pigeon_application::commands::retry_attempt::RetryAttempt;
use pigeon_application::commands::send_test_event::{SendTestEvent, SendTestEventResult};
use pigeon_application::queries::list_audit_log::ListAuditLog;
use pigeon_application::error::ApplicationError;
use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
use pigeon_application::ports::stores::{OidcConfigReadStore, OrganizationReadStore};
use pigeon_application::ports::message_status::MessageWithStatus;
use pigeon_application::ports::stats_read_store::AppStats;
use pigeon_application::queries::get_app_stats::GetAppStats;
use pigeon_application::queries::get_endpoint_stats::GetEndpointStats;
use pigeon_application::queries::get_event_type_stats::GetEventTypeStats;
use pigeon_application::queries::get_dead_letter_by_id::GetDeadLetterById;
use pigeon_application::queries::get_message_by_id::GetMessageById;
use pigeon_application::queries::get_oidc_config_by_id::GetOidcConfigById;
use pigeon_application::queries::list_attempts_by_message::ListAttemptsByMessage;
use pigeon_application::queries::list_dead_letters_by_app::ListDeadLettersByApp;
use pigeon_application::queries::list_messages_by_app::ListMessagesByApp;
use pigeon_application::queries::list_oidc_configs_by_org::ListOidcConfigsByOrg;
use pigeon_application::queries::PaginatedResult;
use pigeon_domain::attempt::Attempt;
use pigeon_domain::dead_letter::DeadLetter;
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId};
use pigeon_domain::organization::{Organization, OrganizationId};
use uuid::Uuid;

use crate::auth::{AuthContext, JwksProvider};

// --- Stub ListAuditLogHandler ---

pub(crate) struct StubListAuditLogHandler;
#[async_trait]
impl QueryHandler<ListAuditLog> for StubListAuditLogHandler {
    async fn handle(&self, _q: ListAuditLog) -> Result<PaginatedResult<pigeon_application::ports::audit_read_store::AuditLogEntry>, ApplicationError> {
        Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 })
    }
}

// --- Stub AuditStore ---

pub(crate) struct StubAuditStore;
#[async_trait]
impl pigeon_application::ports::audit_store::AuditStore for StubAuditStore {
    async fn record(&self, _entry: pigeon_application::ports::audit_store::AuditEntry) -> Result<(), ApplicationError> {
        Ok(())
    }
}

// --- Stub message/attempt/dead-letter query handlers ---

pub(crate) struct StubGetAppStatsHandler;
#[async_trait]
impl QueryHandler<GetAppStats> for StubGetAppStatsHandler {
    async fn handle(&self, _q: GetAppStats) -> Result<AppStats, ApplicationError> {
        Ok(AppStats {
            total_messages: 0,
            total_attempts: 0,
            total_pending: 0,
            total_succeeded: 0,
            total_failed: 0,
            total_dead_lettered: 0,
            success_rate: 0.0,
            time_series: vec![],
        })
    }
}

pub(crate) struct StubGetEventTypeStatsHandler;
#[async_trait]
impl QueryHandler<GetEventTypeStats> for StubGetEventTypeStatsHandler {
    async fn handle(&self, _q: GetEventTypeStats) -> Result<pigeon_application::ports::event_type_stats_read_store::EventTypeStats, ApplicationError> {
        Ok(pigeon_application::ports::event_type_stats_read_store::EventTypeStats {
            total_messages: 0,
            total_attempts: 0,
            total_pending: 0,
            total_succeeded: 0,
            total_failed: 0,
            total_dead_lettered: 0,
            success_rate: 0.0,
            subscribed_endpoints: 0,
            time_series: vec![],
            recent_messages: vec![],
        })
    }
}

pub(crate) struct StubGetEndpointStatsHandler;
#[async_trait]
impl QueryHandler<GetEndpointStats> for StubGetEndpointStatsHandler {
    async fn handle(&self, _q: GetEndpointStats) -> Result<pigeon_application::ports::endpoint_stats_read_store::EndpointStats, ApplicationError> {
        Ok(pigeon_application::ports::endpoint_stats_read_store::EndpointStats {
            total_attempts: 0,
            total_pending: 0,
            total_succeeded: 0,
            total_failed: 0,
            total_dead_lettered: 0,
            success_rate: 0.0,
            consecutive_failures: 0,
            last_delivery_at: None,
            last_status: None,
            time_series: vec![],
        })
    }
}

pub(crate) struct StubGetMessageHandler;
#[async_trait]
impl QueryHandler<GetMessageById> for StubGetMessageHandler {
    async fn handle(&self, _q: GetMessageById) -> Result<Option<MessageWithStatus>, ApplicationError> {
        Ok(None)
    }
}

pub(crate) struct StubListMessagesHandler;
#[async_trait]
impl QueryHandler<ListMessagesByApp> for StubListMessagesHandler {
    async fn handle(
        &self,
        _q: ListMessagesByApp,
    ) -> Result<PaginatedResult<MessageWithStatus>, ApplicationError> {
        Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 })
    }
}

pub(crate) struct StubListAttemptsHandler;
#[async_trait]
impl QueryHandler<ListAttemptsByMessage> for StubListAttemptsHandler {
    async fn handle(&self, _q: ListAttemptsByMessage) -> Result<Vec<Attempt>, ApplicationError> {
        Ok(vec![])
    }
}

pub(crate) struct StubGetDeadLetterHandler;
#[async_trait]
impl QueryHandler<GetDeadLetterById> for StubGetDeadLetterHandler {
    async fn handle(&self, _q: GetDeadLetterById) -> Result<Option<DeadLetter>, ApplicationError> {
        Ok(None)
    }
}

pub(crate) struct StubListDeadLettersHandler;
#[async_trait]
impl QueryHandler<ListDeadLettersByApp> for StubListDeadLettersHandler {
    async fn handle(
        &self,
        _q: ListDeadLettersByApp,
    ) -> Result<PaginatedResult<DeadLetter>, ApplicationError> {
        Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 })
    }
}

// --- Stub OIDC config handlers ---

pub(crate) struct StubCreateOidcConfigHandler;
#[async_trait]
impl CommandHandler<CreateOidcConfig> for StubCreateOidcConfigHandler {
    async fn handle(&self, _c: CreateOidcConfig, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<OidcConfig, ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}

pub(crate) struct StubDeleteOidcConfigHandler;
#[async_trait]
impl CommandHandler<DeleteOidcConfig> for StubDeleteOidcConfigHandler {
    async fn handle(&self, _c: DeleteOidcConfig, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<(), ApplicationError> {
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

pub(crate) struct StubOrganizationReadStore;
#[async_trait]
impl OrganizationReadStore for StubOrganizationReadStore {
    async fn find_by_id(
        &self,
        _id: &OrganizationId,
    ) -> Result<Option<Organization>, ApplicationError> {
        Ok(None)
    }
    async fn find_by_slug(
        &self,
        _slug: &str,
    ) -> Result<Option<Organization>, ApplicationError> {
        Ok(None)
    }
    async fn list(
        &self,
        _offset: u64,
        _limit: u64,
    ) -> Result<Vec<Organization>, ApplicationError> {
        Ok(vec![])
    }
    async fn count(&self) -> Result<u64, ApplicationError> {
        Ok(0)
    }
}

/// Test middleware that injects an AuthContext from the x-org-id header.
/// Test-only header; production uses JWT auth.
pub(crate) async fn test_auth_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let org_id = request
        .headers()
        .get("x-org-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(OrganizationId::from_uuid)
        .unwrap_or_else(OrganizationId::new);

    let auth_context = AuthContext {
        org_id,
        user_id: "test-user".to_string(),
    };
    request.extensions_mut().insert(auth_context);
    next.run(request).await
}

pub(crate) struct StubReplayDeadLetterHandler;
#[async_trait]
impl CommandHandler<ReplayDeadLetter> for StubReplayDeadLetterHandler {
    async fn handle(&self, _c: ReplayDeadLetter, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<DeadLetter, ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}

pub(crate) struct StubRetryAttemptHandler;
#[async_trait]
impl CommandHandler<RetryAttempt> for StubRetryAttemptHandler {
    async fn handle(&self, _c: RetryAttempt, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Attempt, ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}

pub(crate) struct StubRetriggerMessageHandler;
#[async_trait]
impl CommandHandler<RetriggerMessage> for StubRetriggerMessageHandler {
    async fn handle(&self, _c: RetriggerMessage, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<pigeon_application::commands::retrigger_message::RetriggerMessageResult, ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}

pub(crate) struct StubSendTestEventHandler;
#[async_trait]
impl CommandHandler<SendTestEvent> for StubSendTestEventHandler {
    async fn handle(&self, _c: SendTestEvent, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<SendTestEventResult, ApplicationError> {
        Err(ApplicationError::Internal("stub".into()))
    }
}
