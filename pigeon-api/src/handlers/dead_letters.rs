use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::commands::replay_dead_letter::ReplayDeadLetter;
use pigeon_application::queries::get_dead_letter_by_id::GetDeadLetterById;
use pigeon_application::queries::list_dead_letters_by_app::ListDeadLettersByApp;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::dead_letter::DeadLetterId;

use crate::dto::dead_letter::DeadLetterResponse;
use crate::dto::pagination::DeadLetterListQuery;
use crate::error::{ApiError, ErrorBody};
use crate::extractors::{AuthInfo, OrgId};
use crate::state::AppState;
use pigeon_application::mediator::dispatcher::dispatch;

#[derive(Serialize, ToSchema)]
pub(crate) struct ReplayDeadLetterResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub endpoint_id: Uuid,
    pub replayed_at: String,
}

/// List dead letters for an application
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/dead-letters",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        DeadLetterListQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of dead letters"),
    ),
    tag = "dead-letters"
)]
pub(crate) async fn list_dead_letters(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(app_id): Path<Uuid>,
    Query(query): Query<DeadLetterListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);

    let result = state
        .list_dead_letters
        .handle(ListDeadLettersByApp {
            app_id,
            org_id,
            endpoint_id: query.endpoint_id.map(pigeon_domain::endpoint::EndpointId::from_uuid),
            replayed: query.replayed,
            offset: query.offset.unwrap_or(0),
            limit: query.limit.unwrap_or(20),
        })
        .await
        .map_err(ApiError)?;

    let response = serde_json::json!({
        "items": result.items.into_iter().map(DeadLetterResponse::from).collect::<Vec<_>>(),
        "total": result.total,
        "offset": result.offset,
        "limit": result.limit,
    });

    Ok(Json(response))
}

/// Get a dead letter by ID
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/dead-letters/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Dead Letter ID"),
    ),
    responses(
        (status = 200, description = "Dead letter found", body = DeadLetterResponse),
        (status = 404, description = "Dead letter not found", body = ErrorBody),
    ),
    tag = "dead-letters"
)]
pub(crate) async fn get_dead_letter(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let _app_id = ApplicationId::from_uuid(app_id);

    let dl = state
        .get_dead_letter
        .handle(GetDeadLetterById {
            id: DeadLetterId::from_uuid(id),
            org_id,
        })
        .await
        .map_err(ApiError)?
        .ok_or(ApiError(pigeon_application::error::ApplicationError::NotFound))?;

    Ok(Json(DeadLetterResponse::from(dl)))
}

/// Replay a dead-lettered message, creating a new delivery attempt
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/dead-letters/{id}/replay",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Dead letter ID"),
    ),
    responses(
        (status = 200, description = "Dead letter replayed", body = ReplayDeadLetterResponse),
        (status = 404, description = "Dead letter not found", body = ErrorBody),
        (status = 400, description = "Already replayed", body = ErrorBody),
    ),
    tag = "dead-letters"
)]
pub(crate) async fn replay(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path((_app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let command = ReplayDeadLetter {
        org_id: auth.org_id.clone(),
        dead_letter_id: DeadLetterId::from_uuid(id),
    };

    let dead_letter = dispatch(state.replay_dead_letter.clone(), command, &auth.user_id, &auth.org_id, state.uow_factory.clone(), state.audit_store.clone()).await.map_err(ApiError)?;

    let response = ReplayDeadLetterResponse {
        id: *dead_letter.id().as_uuid(),
        message_id: *dead_letter.message_id().as_uuid(),
        endpoint_id: *dead_letter.endpoint_id().as_uuid(),
        replayed_at: dead_letter
            .replayed_at()
            .map(|t| t.to_rfc3339())
            .unwrap_or_default(),
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;
    use pigeon_application::commands::create_application::CreateApplication;
    use pigeon_application::commands::create_endpoint::CreateEndpoint;
    use pigeon_application::commands::create_event_type::CreateEventType;
    use pigeon_application::commands::delete_application::DeleteApplication;
    use pigeon_application::commands::delete_endpoint::DeleteEndpoint;
    use pigeon_application::commands::delete_event_type::DeleteEventType;
    use pigeon_application::commands::send_message::SendMessage;
    use pigeon_application::commands::update_application::UpdateApplication;
    use pigeon_application::commands::update_endpoint::UpdateEndpoint;
    use pigeon_application::commands::update_event_type::UpdateEventType;
    use pigeon_application::commands::create_organization::CreateOrganization;
    use pigeon_application::commands::delete_organization::DeleteOrganization;
    use pigeon_application::commands::update_organization::UpdateOrganization;
    use pigeon_application::error::ApplicationError;
    use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
    use pigeon_application::queries::get_application_by_id::GetApplicationById;
    use pigeon_application::queries::get_endpoint_by_id::GetEndpointById;
    use pigeon_application::queries::get_event_type_by_id::GetEventTypeById;
    use pigeon_application::queries::get_organization_by_id::GetOrganizationById;
    use pigeon_application::queries::list_applications::ListApplications;
    use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp;
    use pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp;
    use pigeon_application::queries::list_organizations::ListOrganizations;
    use pigeon_application::queries::PaginatedResult;
    use pigeon_application::commands::send_message::SendMessageResult;
    use pigeon_domain::application::Application;
    use pigeon_domain::dead_letter::DeadLetter;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::organization::Organization;
    use std::sync::Arc;
    use tower::ServiceExt;

    // --- Stubs for existing handlers (minimal, return errors/empty) ---

    struct StubCreateHandler;
    #[async_trait]
    impl CommandHandler<CreateApplication> for StubCreateHandler {
        async fn handle(&self, _c: CreateApplication, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Application, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateHandler;
    #[async_trait]
    impl CommandHandler<UpdateApplication> for StubUpdateHandler {
        async fn handle(&self, _c: UpdateApplication, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Application, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteHandler;
    #[async_trait]
    impl CommandHandler<DeleteApplication> for StubDeleteHandler {
        async fn handle(&self, _c: DeleteApplication, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<(), ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubGetHandler;
    #[async_trait]
    impl QueryHandler<GetApplicationById> for StubGetHandler {
        async fn handle(&self, _q: GetApplicationById) -> Result<Option<Application>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListHandler;
    #[async_trait]
    impl QueryHandler<ListApplications> for StubListHandler {
        async fn handle(&self, _q: ListApplications) -> Result<PaginatedResult<Application>, ApplicationError> {
            Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 })
        }
    }

    struct StubCreateEtHandler;
    #[async_trait]
    impl CommandHandler<CreateEventType> for StubCreateEtHandler {
        async fn handle(&self, _c: CreateEventType, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<EventType, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateEtHandler;
    #[async_trait]
    impl CommandHandler<UpdateEventType> for StubUpdateEtHandler {
        async fn handle(&self, _c: UpdateEventType, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<EventType, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteEtHandler;
    #[async_trait]
    impl CommandHandler<DeleteEventType> for StubDeleteEtHandler {
        async fn handle(&self, _c: DeleteEventType, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<(), ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubGetEtHandler;
    #[async_trait]
    impl QueryHandler<GetEventTypeById> for StubGetEtHandler {
        async fn handle(&self, _q: GetEventTypeById) -> Result<Option<EventType>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListEtHandler;
    #[async_trait]
    impl QueryHandler<ListEventTypesByApp> for StubListEtHandler {
        async fn handle(&self, _q: ListEventTypesByApp) -> Result<PaginatedResult<EventType>, ApplicationError> {
            Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 })
        }
    }

    struct StubCreateEpHandler;
    #[async_trait]
    impl CommandHandler<CreateEndpoint> for StubCreateEpHandler {
        async fn handle(&self, _c: CreateEndpoint, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Endpoint, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateEpHandler;
    #[async_trait]
    impl CommandHandler<UpdateEndpoint> for StubUpdateEpHandler {
        async fn handle(&self, _c: UpdateEndpoint, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Endpoint, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteEpHandler;
    #[async_trait]
    impl CommandHandler<DeleteEndpoint> for StubDeleteEpHandler {
        async fn handle(&self, _c: DeleteEndpoint, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<(), ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubGetEpHandler;
    #[async_trait]
    impl QueryHandler<GetEndpointById> for StubGetEpHandler {
        async fn handle(&self, _q: GetEndpointById) -> Result<Option<Endpoint>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListEpHandler;
    #[async_trait]
    impl QueryHandler<ListEndpointsByApp> for StubListEpHandler {
        async fn handle(&self, _q: ListEndpointsByApp) -> Result<PaginatedResult<Endpoint>, ApplicationError> {
            Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 })
        }
    }

    struct StubHealthChecker;
    #[async_trait]
    impl pigeon_application::ports::health::HealthChecker for StubHealthChecker {
        async fn check(&self) -> bool {
            true
        }
    }

    struct StubCreateOrgHandler;
    #[async_trait]
    impl CommandHandler<CreateOrganization> for StubCreateOrgHandler {
        async fn handle(&self, _c: CreateOrganization, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Organization, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateOrgHandler;
    #[async_trait]
    impl CommandHandler<UpdateOrganization> for StubUpdateOrgHandler {
        async fn handle(&self, _c: UpdateOrganization, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Organization, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteOrgHandler;
    #[async_trait]
    impl CommandHandler<DeleteOrganization> for StubDeleteOrgHandler {
        async fn handle(&self, _c: DeleteOrganization, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<(), ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubGetOrgHandler;
    #[async_trait]
    impl QueryHandler<GetOrganizationById> for StubGetOrgHandler {
        async fn handle(&self, _q: GetOrganizationById) -> Result<Option<Organization>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListOrgsHandler;
    #[async_trait]
    impl QueryHandler<ListOrganizations> for StubListOrgsHandler {
        async fn handle(&self, _q: ListOrganizations) -> Result<PaginatedResult<Organization>, ApplicationError> {
            Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 })
        }
    }

    struct StubSendMessageHandler;
    #[async_trait]
    impl CommandHandler<SendMessage> for StubSendMessageHandler {
        async fn handle(&self, _c: SendMessage, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<SendMessageResult, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    // --- Dead-letter specific fakes ---

    struct StubListDeadLettersHandler;
    #[async_trait]
    impl QueryHandler<ListDeadLettersByApp> for StubListDeadLettersHandler {
        async fn handle(&self, _q: ListDeadLettersByApp) -> Result<PaginatedResult<DeadLetter>, ApplicationError> {
            Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 })
        }
    }

    struct StubGetDeadLetterHandler {
        dead_letter: Option<DeadLetter>,
    }
    #[async_trait]
    impl QueryHandler<GetDeadLetterById> for StubGetDeadLetterHandler {
        async fn handle(&self, _q: GetDeadLetterById) -> Result<Option<DeadLetter>, ApplicationError> {
            Ok(self.dead_letter.clone())
        }
    }

    struct FakeReplayHandler {
        result: Result<DeadLetter, ApplicationError>,
    }
    #[async_trait]
    impl CommandHandler<ReplayDeadLetter> for FakeReplayHandler {
        async fn handle(&self, _c: ReplayDeadLetter, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<DeadLetter, ApplicationError> {
            match &self.result {
                Ok(dl) => Ok(dl.clone()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    // --- Helpers ---

    const TEST_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

    fn build_state(
        get_dead_letter: impl QueryHandler<GetDeadLetterById> + 'static,
        list_dead_letters: impl QueryHandler<ListDeadLettersByApp> + 'static,
        replay_dead_letter: impl CommandHandler<ReplayDeadLetter> + 'static,
    ) -> AppState {
        use crate::test_support::*;
        AppState {
            create_application: Arc::new(StubCreateHandler),
            update_application: Arc::new(StubUpdateHandler),
            delete_application: Arc::new(StubDeleteHandler),
            get_application: Arc::new(StubGetHandler),
            list_applications: Arc::new(StubListHandler),
            send_message: Arc::new(StubSendMessageHandler),
            create_event_type: Arc::new(StubCreateEtHandler),
            update_event_type: Arc::new(StubUpdateEtHandler),
            delete_event_type: Arc::new(StubDeleteEtHandler),
            get_event_type: Arc::new(StubGetEtHandler),
            list_event_types: Arc::new(StubListEtHandler),
            create_endpoint: Arc::new(StubCreateEpHandler),
            update_endpoint: Arc::new(StubUpdateEpHandler),
            delete_endpoint: Arc::new(StubDeleteEpHandler),
            get_endpoint: Arc::new(StubGetEpHandler),
            list_endpoints: Arc::new(StubListEpHandler),
            get_app_stats: Arc::new(StubGetAppStatsHandler),
            get_event_type_stats: Arc::new(StubGetEventTypeStatsHandler),
            get_endpoint_stats: Arc::new(StubGetEndpointStatsHandler),
            get_message: Arc::new(StubGetMessageHandler),
            list_messages: Arc::new(StubListMessagesHandler),
            list_attempts: Arc::new(StubListAttemptsHandler),
            get_dead_letter: Arc::new(get_dead_letter),
            list_dead_letters: Arc::new(list_dead_letters),
            health_checker: Arc::new(StubHealthChecker),
            create_organization: Arc::new(StubCreateOrgHandler),
            update_organization: Arc::new(StubUpdateOrgHandler),
            delete_organization: Arc::new(StubDeleteOrgHandler),
            get_organization: Arc::new(StubGetOrgHandler),
            list_organizations: Arc::new(StubListOrgsHandler),
            create_oidc_config: Arc::new(StubCreateOidcConfigHandler),
            delete_oidc_config: Arc::new(StubDeleteOidcConfigHandler),
            get_oidc_config: Arc::new(StubGetOidcConfigHandler),
            list_oidc_configs: Arc::new(StubListOidcConfigsHandler),
            oidc_config_read_store: Arc::new(StubOidcConfigReadStore),
            org_read_store: Arc::new(StubOrganizationReadStore),
            jwks_provider: Arc::new(StubJwksProvider),
            replay_dead_letter: Arc::new(replay_dead_letter),
            retry_attempt: Arc::new(StubRetryAttemptHandler),
            retrigger_message: Arc::new(StubRetriggerMessageHandler),
            send_test_event: Arc::new(StubSendTestEventHandler),
            rotate_signing_secret: Arc::new(StubRotateSigningSecretHandler),
            revoke_signing_secret: Arc::new(StubRevokeSigningSecretHandler),
            list_audit_log: Arc::new(StubListAuditLogHandler),
            audit_store: Arc::new(StubAuditStore),
            uow_factory: Arc::new(pigeon_application::test_support::fakes::FakeUnitOfWorkFactory::new(pigeon_application::test_support::fakes::OperationLog::new())),
            metrics_render: Arc::new(|| String::new()),
            admin_org_id: None,
        }
    }

    fn test_router(state: AppState) -> Router {
        crate::router_without_auth(state)
    }

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    // --- Tests ---

    #[tokio::test]
    async fn list_dead_letters_returns_200() {
        let app_id = ApplicationId::new();
        let state = build_state(
            StubGetDeadLetterHandler { dead_letter: None },
            StubListDeadLettersHandler,
            crate::test_support::StubReplayDeadLetterHandler,
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/applications/{}/dead-letters", app_id.as_uuid()))
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response.into_body()).await;
        assert!(json["items"].is_array());
    }

    #[tokio::test]
    async fn get_dead_letter_not_found_returns_404() {
        let app_id = ApplicationId::new();
        let state = build_state(
            StubGetDeadLetterHandler { dead_letter: None },
            StubListDeadLettersHandler,
            crate::test_support::StubReplayDeadLetterHandler,
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!(
                        "/api/v1/applications/{}/dead-letters/{}",
                        app_id.as_uuid(),
                        Uuid::new_v4()
                    ))
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn replay_returns_200() {
        let app_id = ApplicationId::new();
        let mut dl = pigeon_domain::test_support::any_dead_letter();
        dl.mark_replayed();

        let state = build_state(
            StubGetDeadLetterHandler { dead_letter: None },
            StubListDeadLettersHandler,
            FakeReplayHandler { result: Ok(dl.clone()) },
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/v1/applications/{}/dead-letters/{}/replay",
                        app_id.as_uuid(),
                        dl.id().as_uuid()
                    ))
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response.into_body()).await;
        assert!(json["id"].is_string());
        assert!(json["replayed_at"].is_string());
    }

    #[tokio::test]
    async fn replay_not_found_returns_404() {
        let app_id = ApplicationId::new();
        let state = build_state(
            StubGetDeadLetterHandler { dead_letter: None },
            StubListDeadLettersHandler,
            FakeReplayHandler { result: Err(ApplicationError::NotFound) },
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/v1/applications/{}/dead-letters/{}/replay",
                        app_id.as_uuid(),
                        Uuid::new_v4()
                    ))
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
