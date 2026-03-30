use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use pigeon_application::commands::create_endpoint::CreateEndpoint;
use pigeon_application::commands::delete_endpoint::DeleteEndpoint;
use pigeon_application::commands::update_endpoint::UpdateEndpoint;
use pigeon_application::queries::get_endpoint_by_id::GetEndpointById;
use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::version::Version;

use crate::dto::endpoint::{CreateEndpointRequest, EndpointResponse, UpdateEndpointRequest};
use crate::dto::pagination::ListQuery;
use crate::error::{ApiError, ErrorBody};
use crate::extractors::{AuthInfo, OrgId};
use crate::state::AppState;
use pigeon_application::mediator::dispatcher::dispatch;

/// Create a new endpoint
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/endpoints",
    params(("app_id" = Uuid, Path, description = "Application ID")),
    request_body = CreateEndpointRequest,
    responses(
        (status = 201, description = "Endpoint created", body = EndpointResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
    tag = "endpoints"
)]
pub(crate) async fn create_endpoint(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path(app_id): Path<Uuid>,
    Json(body): Json<CreateEndpointRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);

    let command = CreateEndpoint {
        org_id: auth.org_id.clone(),
        app_id,
        name: body.name,
        url: body.url,
        signing_secret: body.signing_secret,
        event_type_ids: body.event_type_ids.into_iter().map(EventTypeId::from_uuid).collect(),
    };

    let ep = dispatch(state.create_endpoint.clone(), command, &auth.user_id, &auth.org_id, state.uow_factory.clone(), state.audit_store.clone()).await.map_err(ApiError)?;
    let response = EndpointResponse::from(ep);

    Ok((StatusCode::CREATED, Json(response)))
}

/// List endpoints for an application
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/endpoints",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ListQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of endpoints"),
    ),
    tag = "endpoints"
)]
pub(crate) async fn list_endpoints(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(app_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);

    let list_query = ListEndpointsByApp {
        app_id,
        org_id,
        offset: query.offset.unwrap_or(0),
        limit: query.limit.unwrap_or(20),
    };

    let result = state.list_endpoints.handle(list_query).await.map_err(ApiError)?;

    let response = serde_json::json!({
        "items": result.items.into_iter().map(EndpointResponse::from).collect::<Vec<_>>(),
        "total": result.total,
        "offset": result.offset,
        "limit": result.limit,
    });

    Ok(Json(response))
}

/// Get an endpoint by ID
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/endpoints/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Endpoint ID"),
    ),
    responses(
        (status = 200, description = "Endpoint found", body = EndpointResponse),
        (status = 404, description = "Endpoint not found", body = ErrorBody),
    ),
    tag = "endpoints"
)]
pub(crate) async fn get_endpoint(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let _app_id = ApplicationId::from_uuid(app_id);

    let query = GetEndpointById {
        id: EndpointId::from_uuid(id),
        org_id,
    };

    let ep = state
        .get_endpoint
        .handle(query)
        .await
        .map_err(ApiError)?
        .ok_or(ApiError(pigeon_application::error::ApplicationError::NotFound))?;

    Ok(Json(EndpointResponse::from(ep)))
}

/// Update an endpoint
#[utoipa::path(
    put,
    path = "/api/v1/applications/{app_id}/endpoints/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Endpoint ID"),
    ),
    request_body = UpdateEndpointRequest,
    responses(
        (status = 200, description = "Endpoint updated", body = EndpointResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 404, description = "Endpoint not found", body = ErrorBody),
        (status = 409, description = "Version conflict", body = ErrorBody),
    ),
    tag = "endpoints"
)]
pub(crate) async fn update_endpoint(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateEndpointRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let _app_id = ApplicationId::from_uuid(app_id);

    let command = UpdateEndpoint {
        org_id: auth.org_id.clone(),
        id: EndpointId::from_uuid(id),
        url: body.url,
        signing_secret: body.signing_secret,
        event_type_ids: body.event_type_ids.into_iter().map(EventTypeId::from_uuid).collect(),
        version: Version::new(body.version),
    };

    let ep = dispatch(state.update_endpoint.clone(), command, &auth.user_id, &auth.org_id, state.uow_factory.clone(), state.audit_store.clone()).await.map_err(ApiError)?;

    Ok(Json(EndpointResponse::from(ep)))
}

/// Delete an endpoint
#[utoipa::path(
    delete,
    path = "/api/v1/applications/{app_id}/endpoints/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Endpoint ID"),
    ),
    responses(
        (status = 204, description = "Endpoint deleted"),
        (status = 404, description = "Endpoint not found", body = ErrorBody),
    ),
    tag = "endpoints"
)]
pub(crate) async fn delete_endpoint(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let _app_id = ApplicationId::from_uuid(app_id);

    let command = DeleteEndpoint {
        org_id: auth.org_id.clone(),
        id: EndpointId::from_uuid(id),
    };

    dispatch(state.delete_endpoint.clone(), command, &auth.user_id, &auth.org_id, state.uow_factory.clone(), state.audit_store.clone()).await.map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;
    use pigeon_application::commands::create_application::CreateApplication;
    use pigeon_application::mediator::pipeline::RequestContext;
    use pigeon_application::commands::create_event_type::CreateEventType;
    use pigeon_application::commands::delete_application::DeleteApplication;
    use pigeon_application::commands::delete_event_type::DeleteEventType;
    use pigeon_application::commands::send_message::{SendMessage, SendMessageResult};
    use pigeon_application::commands::update_application::UpdateApplication;
    use pigeon_application::commands::update_event_type::UpdateEventType;
    use pigeon_application::error::ApplicationError;
    use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
    use pigeon_application::queries::get_application_by_id::GetApplicationById;
    use pigeon_application::queries::get_event_type_by_id::GetEventTypeById;
    use pigeon_application::queries::list_applications::ListApplications;
    use pigeon_application::commands::create_organization::CreateOrganization;
    use pigeon_application::commands::delete_organization::DeleteOrganization;
    use pigeon_application::commands::update_organization::UpdateOrganization;
    use pigeon_application::queries::get_organization_by_id::GetOrganizationById;
    use pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp;
    use pigeon_application::queries::list_organizations::ListOrganizations;
    use pigeon_application::queries::PaginatedResult;
    use pigeon_domain::application::Application;
    use pigeon_domain::endpoint::{Endpoint, EndpointState};
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::organization::Organization;
    use std::sync::Arc;
    use tower::ServiceExt;

    // --- Stubs for existing handlers ---

    struct StubCreateAppHandler;
    #[async_trait]
    impl CommandHandler<CreateApplication> for StubCreateAppHandler {
        async fn handle(&self, _c: CreateApplication, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Application, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateAppHandler;
    #[async_trait]
    impl CommandHandler<UpdateApplication> for StubUpdateAppHandler {
        async fn handle(&self, _c: UpdateApplication, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<Application, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteAppHandler;
    #[async_trait]
    impl CommandHandler<DeleteApplication> for StubDeleteAppHandler {
        async fn handle(&self, _c: DeleteApplication, _ctx: &mut pigeon_application::mediator::pipeline::RequestContext) -> Result<(), ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubGetAppHandler;
    #[async_trait]
    impl QueryHandler<GetApplicationById> for StubGetAppHandler {
        async fn handle(&self, _q: GetApplicationById) -> Result<Option<Application>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListAppsHandler;
    #[async_trait]
    impl QueryHandler<ListApplications> for StubListAppsHandler {
        async fn handle(&self, _q: ListApplications) -> Result<PaginatedResult<Application>, ApplicationError> {
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

    // --- Endpoint fakes ---

    struct FakeCreateEpHandler {
        result: Result<Endpoint, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<CreateEndpoint> for FakeCreateEpHandler {
        async fn handle(&self, command: CreateEndpoint, _ctx: &mut RequestContext) -> Result<Endpoint, ApplicationError> {
            match &self.result {
                Ok(_) => Endpoint::new(
                    command.app_id,
                    command.name,
                    command.url,
                    command.signing_secret,
                    command.event_type_ids,
                )
                .map_err(|e| ApplicationError::Validation(e.to_string())),
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeUpdateEpHandler {
        result: Result<Endpoint, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<UpdateEndpoint> for FakeUpdateEpHandler {
        async fn handle(&self, _command: UpdateEndpoint, _ctx: &mut RequestContext) -> Result<Endpoint, ApplicationError> {
            match &self.result {
                Ok(ep) => Ok(ep.clone()),
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    ApplicationError::Conflict => ApplicationError::Conflict,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeDeleteEpHandler {
        result: Result<(), ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<DeleteEndpoint> for FakeDeleteEpHandler {
        async fn handle(&self, _command: DeleteEndpoint, _ctx: &mut RequestContext) -> Result<(), ApplicationError> {
            match &self.result {
                Ok(()) => Ok(()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeGetEpHandler {
        result: Result<Option<Endpoint>, ApplicationError>,
    }

    #[async_trait]
    impl QueryHandler<GetEndpointById> for FakeGetEpHandler {
        async fn handle(&self, _query: GetEndpointById) -> Result<Option<Endpoint>, ApplicationError> {
            match &self.result {
                Ok(ep) => Ok(ep.clone()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeListEpHandler {
        result: Result<PaginatedResult<Endpoint>, ApplicationError>,
    }

    #[async_trait]
    impl QueryHandler<ListEndpointsByApp> for FakeListEpHandler {
        async fn handle(&self, _query: ListEndpointsByApp) -> Result<PaginatedResult<Endpoint>, ApplicationError> {
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(ApplicationError::Internal(e.to_string())),
            }
        }
    }

    // --- Helpers ---

    fn org_header() -> &'static str {
        "00000000-0000-0000-0000-000000000001"
    }

    fn fake_endpoint() -> Endpoint {
        Endpoint::reconstitute(EndpointState::fake())
    }

    fn build_ep_state(
        create: FakeCreateEpHandler,
        update: FakeUpdateEpHandler,
        delete: FakeDeleteEpHandler,
        get: FakeGetEpHandler,
        list: FakeListEpHandler,
    ) -> AppState {
        use crate::test_support::*;
        AppState {
            create_application: Arc::new(StubCreateAppHandler),
            update_application: Arc::new(StubUpdateAppHandler),
            delete_application: Arc::new(StubDeleteAppHandler),
            send_message: Arc::new(StubSendMessageHandler),
            get_application: Arc::new(StubGetAppHandler),
            list_applications: Arc::new(StubListAppsHandler),
            create_event_type: Arc::new(StubCreateEtHandler),
            update_event_type: Arc::new(StubUpdateEtHandler),
            delete_event_type: Arc::new(StubDeleteEtHandler),
            get_event_type: Arc::new(StubGetEtHandler),
            list_event_types: Arc::new(StubListEtHandler),
            create_endpoint: Arc::new(create),
            update_endpoint: Arc::new(update),
            delete_endpoint: Arc::new(delete),
            get_endpoint: Arc::new(get),
            list_endpoints: Arc::new(list),
            get_app_stats: Arc::new(StubGetAppStatsHandler),
            get_event_type_stats: Arc::new(StubGetEventTypeStatsHandler),
            get_endpoint_stats: Arc::new(StubGetEndpointStatsHandler),
            get_message: Arc::new(StubGetMessageHandler),
            list_messages: Arc::new(StubListMessagesHandler),
            list_attempts: Arc::new(StubListAttemptsHandler),
            get_dead_letter: Arc::new(StubGetDeadLetterHandler),
            list_dead_letters: Arc::new(StubListDeadLettersHandler),
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
            replay_dead_letter: Arc::new(StubReplayDeadLetterHandler),
            retry_attempt: Arc::new(StubRetryAttemptHandler),
            retrigger_message: Arc::new(StubRetriggerMessageHandler),
            send_test_event: Arc::new(StubSendTestEventHandler),
            list_audit_log: Arc::new(StubListAuditLogHandler),
            audit_store: Arc::new(StubAuditStore),
            uow_factory: Arc::new(pigeon_application::test_support::fakes::FakeUnitOfWorkFactory::new(pigeon_application::test_support::fakes::OperationLog::new())),
            metrics_render: Arc::new(|| String::new()),
            admin_org_id: None,
        }
    }

    fn default_ep_state(ep: Endpoint) -> AppState {
        build_ep_state(
            FakeCreateEpHandler { result: Ok(ep.clone()) },
            FakeUpdateEpHandler { result: Ok(ep.clone()) },
            FakeDeleteEpHandler { result: Ok(()) },
            FakeGetEpHandler { result: Ok(Some(ep.clone())) },
            FakeListEpHandler {
                result: Ok(PaginatedResult {
                    items: vec![ep],
                    total: 1,
                    offset: 0,
                    limit: 20,
                }),
            },
        )
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
    async fn create_returns_201() {
        let ep = fake_endpoint();
        let app_id = ep.app_id().clone();
        let state = default_ep_state(ep);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/applications/{}/endpoints", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", org_header())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "url": "https://example.com/webhook",
                            "signing_secret": "whsec_secret123",
                            "event_type_ids": []
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let json = body_json(response.into_body()).await;
        assert_eq!(json["url"], "https://example.com/webhook");
        // signing_secret should NOT be in response
        assert!(json.get("signing_secret").is_none());
    }

    #[tokio::test]
    async fn create_with_empty_url_returns_400() {
        let ep = fake_endpoint();
        let app_id = ep.app_id().clone();
        let state = default_ep_state(ep);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/applications/{}/endpoints", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", org_header())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "url": "",
                            "signing_secret": "whsec_secret123",
                            "event_type_ids": []
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_existing_returns_200() {
        let ep = fake_endpoint();
        let app_id = ep.app_id().clone();
        let id = *ep.id().as_uuid();
        let state = default_ep_state(ep);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/applications/{}/endpoints/{id}", app_id.as_uuid()))
                    .header("x-org-id", org_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response.into_body()).await;
        assert!(json["id"].is_string());
        assert!(json.get("signing_secret").is_none());
    }

    #[tokio::test]
    async fn get_nonexistent_returns_404() {
        let ep = fake_endpoint();
        let app_id = pigeon_domain::application::ApplicationId::new();
        let id = Uuid::new_v4();
        let state = build_ep_state(
            FakeCreateEpHandler { result: Ok(ep.clone()) },
            FakeUpdateEpHandler { result: Ok(ep.clone()) },
            FakeDeleteEpHandler { result: Ok(()) },
            FakeGetEpHandler { result: Ok(None) },
            FakeListEpHandler {
                result: Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 }),
            },
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/applications/{}/endpoints/{id}", app_id.as_uuid()))
                    .header("x-org-id", org_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_returns_paginated() {
        let ep = fake_endpoint();
        let app_id = ep.app_id().clone();
        let state = default_ep_state(ep);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!(
                        "/api/v1/applications/{}/endpoints?offset=0&limit=10",
                        app_id.as_uuid()
                    ))
                    .header("x-org-id", org_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response.into_body()).await;
        assert!(json["items"].is_array());
        assert_eq!(json["total"], 1);
    }

    #[tokio::test]
    async fn update_returns_200() {
        let ep = fake_endpoint();
        let app_id = ep.app_id().clone();
        let id = *ep.id().as_uuid();
        let state = default_ep_state(ep);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(&format!("/api/v1/applications/{}/endpoints/{id}", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", org_header())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "url": "https://updated.example.com/webhook",
                            "signing_secret": "whsec_new",
                            "event_type_ids": [],
                            "version": 0
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn update_conflict_returns_409() {
        let ep = fake_endpoint();
        let app_id = ep.app_id().clone();
        let id = *ep.id().as_uuid();
        let state = build_ep_state(
            FakeCreateEpHandler { result: Ok(ep.clone()) },
            FakeUpdateEpHandler { result: Err(ApplicationError::Conflict) },
            FakeDeleteEpHandler { result: Ok(()) },
            FakeGetEpHandler { result: Ok(Some(ep.clone())) },
            FakeListEpHandler {
                result: Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 }),
            },
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(&format!("/api/v1/applications/{}/endpoints/{id}", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", org_header())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "url": "https://updated.example.com/webhook",
                            "signing_secret": "whsec_new",
                            "event_type_ids": [],
                            "version": 999
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn delete_returns_204() {
        let ep = fake_endpoint();
        let app_id = ep.app_id().clone();
        let id = *ep.id().as_uuid();
        let state = default_ep_state(ep);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/api/v1/applications/{}/endpoints/{id}", app_id.as_uuid()))
                    .header("x-org-id", org_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_404() {
        let ep = fake_endpoint();
        let app_id = pigeon_domain::application::ApplicationId::new();
        let id = Uuid::new_v4();
        let state = build_ep_state(
            FakeCreateEpHandler { result: Ok(ep.clone()) },
            FakeUpdateEpHandler { result: Ok(ep.clone()) },
            FakeDeleteEpHandler { result: Err(ApplicationError::NotFound) },
            FakeGetEpHandler { result: Ok(Some(ep.clone())) },
            FakeListEpHandler {
                result: Ok(PaginatedResult { items: vec![], total: 0, offset: 0, limit: 20 }),
            },
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/api/v1/applications/{}/endpoints/{id}", app_id.as_uuid()))
                    .header("x-org-id", org_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
