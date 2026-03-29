use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use pigeon_application::commands::create_event_type::CreateEventType;
use pigeon_application::commands::delete_event_type::DeleteEventType;
use pigeon_application::commands::update_event_type::UpdateEventType;
use pigeon_application::queries::get_event_type_by_id::GetEventTypeById;
use pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::version::Version;

use crate::dto::event_type::{CreateEventTypeRequest, EventTypeResponse, UpdateEventTypeRequest};
use crate::dto::pagination::ListQuery;
use crate::error::{ApiError, ErrorBody};
use crate::extractors::OrgId;
use crate::state::AppState;

use super::verify_app_ownership;

/// Create a new event type
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/event-types",
    params(("app_id" = Uuid, Path, description = "Application ID")),
    request_body = CreateEventTypeRequest,
    responses(
        (status = 201, description = "Event type created", body = EventTypeResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
    tag = "event_types"
)]
pub async fn create_event_type(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(app_id): Path<Uuid>,
    Json(body): Json<CreateEventTypeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let command = CreateEventType {
        org_id,
        app_id,
        name: body.name,
        schema: body.schema,
    };

    let et = state.create_event_type.handle(command).await.map_err(ApiError)?;
    let response = EventTypeResponse::from(et);

    Ok((StatusCode::CREATED, Json(response)))
}

/// List event types for an application
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/event-types",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ListQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of event types"),
    ),
    tag = "event_types"
)]
pub async fn list_event_types(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(app_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let list_query = ListEventTypesByApp {
        app_id,
        org_id,
        offset: query.offset.unwrap_or(0),
        limit: query.limit.unwrap_or(20),
    };

    let result = state.list_event_types.handle(list_query).await.map_err(ApiError)?;

    let response = serde_json::json!({
        "items": result.items.into_iter().map(EventTypeResponse::from).collect::<Vec<_>>(),
        "total": result.total,
        "offset": result.offset,
        "limit": result.limit,
    });

    Ok(Json(response))
}

/// Get an event type by ID
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/event-types/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Event Type ID"),
    ),
    responses(
        (status = 200, description = "Event type found", body = EventTypeResponse),
        (status = 404, description = "Event type not found", body = ErrorBody),
    ),
    tag = "event_types"
)]
pub async fn get_event_type(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let query = GetEventTypeById {
        id: EventTypeId::from_uuid(id),
        org_id,
    };

    let et = state
        .get_event_type
        .handle(query)
        .await
        .map_err(ApiError)?
        .ok_or(ApiError(pigeon_application::error::ApplicationError::NotFound))?;

    Ok(Json(EventTypeResponse::from(et)))
}

/// Update an event type
#[utoipa::path(
    put,
    path = "/api/v1/applications/{app_id}/event-types/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Event Type ID"),
    ),
    request_body = UpdateEventTypeRequest,
    responses(
        (status = 200, description = "Event type updated", body = EventTypeResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 404, description = "Event type not found", body = ErrorBody),
        (status = 409, description = "Version conflict", body = ErrorBody),
    ),
    tag = "event_types"
)]
pub async fn update_event_type(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateEventTypeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let command = UpdateEventType {
        org_id,
        id: EventTypeId::from_uuid(id),
        name: body.name,
        schema: body.schema,
        version: Version::new(body.version),
    };

    let et = state.update_event_type.handle(command).await.map_err(ApiError)?;

    Ok(Json(EventTypeResponse::from(et)))
}

/// Delete an event type
#[utoipa::path(
    delete,
    path = "/api/v1/applications/{app_id}/event-types/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Event Type ID"),
    ),
    responses(
        (status = 204, description = "Event type deleted"),
        (status = 404, description = "Event type not found", body = ErrorBody),
    ),
    tag = "event_types"
)]
pub async fn delete_event_type(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let command = DeleteEventType {
        org_id,
        id: EventTypeId::from_uuid(id),
    };

    state.delete_event_type.handle(command).await.map_err(ApiError)?;

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
    use pigeon_application::commands::create_endpoint::CreateEndpoint;
    use pigeon_application::commands::delete_application::DeleteApplication;
    use pigeon_application::commands::delete_endpoint::DeleteEndpoint;
    use pigeon_application::commands::send_message::{SendMessage, SendMessageResult};
    use pigeon_application::commands::update_application::UpdateApplication;
    use pigeon_application::commands::update_endpoint::UpdateEndpoint;
    use pigeon_application::error::ApplicationError;
    use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
    use pigeon_application::queries::get_application_by_id::GetApplicationById;
    use pigeon_application::queries::get_endpoint_by_id::GetEndpointById;
    use pigeon_application::queries::list_applications::ListApplications;
    use pigeon_application::commands::create_organization::CreateOrganization;
    use pigeon_application::commands::delete_organization::DeleteOrganization;
    use pigeon_application::commands::update_organization::UpdateOrganization;
    use pigeon_application::queries::get_organization_by_id::GetOrganizationById;
    use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp;
    use pigeon_application::queries::list_organizations::ListOrganizations;
    use pigeon_application::queries::PaginatedResult;
    use pigeon_domain::application::Application;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::event_type::{EventType, EventTypeState};
    use pigeon_domain::organization::Organization;
    use std::sync::Arc;
    use tower::ServiceExt;

    // --- Stubs for existing handlers ---

    struct StubCreateAppHandler;
    #[async_trait]
    impl CommandHandler<CreateApplication> for StubCreateAppHandler {
        async fn handle(&self, _c: CreateApplication) -> Result<Application, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateAppHandler;
    #[async_trait]
    impl CommandHandler<UpdateApplication> for StubUpdateAppHandler {
        async fn handle(&self, _c: UpdateApplication) -> Result<Application, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteAppHandler;
    #[async_trait]
    impl CommandHandler<DeleteApplication> for StubDeleteAppHandler {
        async fn handle(&self, _c: DeleteApplication) -> Result<(), ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubGetAppHandler;
    #[async_trait]
    impl QueryHandler<GetApplicationById> for StubGetAppHandler {
        async fn handle(
            &self,
            _q: GetApplicationById,
        ) -> Result<Option<Application>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListAppsHandler;
    #[async_trait]
    impl QueryHandler<ListApplications> for StubListAppsHandler {
        async fn handle(
            &self,
            _q: ListApplications,
        ) -> Result<PaginatedResult<Application>, ApplicationError> {
            Ok(PaginatedResult {
                items: vec![],
                total: 0,
                offset: 0,
                limit: 20,
            })
        }
    }

    struct StubSendMessageHandler;
    #[async_trait]
    impl CommandHandler<SendMessage> for StubSendMessageHandler {
        async fn handle(&self, _c: SendMessage) -> Result<SendMessageResult, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubCreateEpHandler;
    #[async_trait]
    impl CommandHandler<CreateEndpoint> for StubCreateEpHandler {
        async fn handle(&self, _c: CreateEndpoint) -> Result<Endpoint, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateEpHandler;
    #[async_trait]
    impl CommandHandler<UpdateEndpoint> for StubUpdateEpHandler {
        async fn handle(&self, _c: UpdateEndpoint) -> Result<Endpoint, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteEpHandler;
    #[async_trait]
    impl CommandHandler<DeleteEndpoint> for StubDeleteEpHandler {
        async fn handle(&self, _c: DeleteEndpoint) -> Result<(), ApplicationError> {
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
        async fn handle(&self, _c: CreateOrganization) -> Result<Organization, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateOrgHandler;
    #[async_trait]
    impl CommandHandler<UpdateOrganization> for StubUpdateOrgHandler {
        async fn handle(&self, _c: UpdateOrganization) -> Result<Organization, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteOrgHandler;
    #[async_trait]
    impl CommandHandler<DeleteOrganization> for StubDeleteOrgHandler {
        async fn handle(&self, _c: DeleteOrganization) -> Result<(), ApplicationError> {
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

    // --- Event type fakes ---

    struct FakeCreateEtHandler {
        result: Result<EventType, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<CreateEventType> for FakeCreateEtHandler {
        async fn handle(&self, command: CreateEventType) -> Result<EventType, ApplicationError> {
            match &self.result {
                Ok(_) => EventType::new(command.app_id, command.name, command.schema)
                    .map_err(|e| ApplicationError::Validation(e.to_string())),
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeUpdateEtHandler {
        result: Result<EventType, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<UpdateEventType> for FakeUpdateEtHandler {
        async fn handle(
            &self,
            _command: UpdateEventType,
        ) -> Result<EventType, ApplicationError> {
            match &self.result {
                Ok(et) => Ok(et.clone()),
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    ApplicationError::Conflict => ApplicationError::Conflict,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeDeleteEtHandler {
        result: Result<(), ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<DeleteEventType> for FakeDeleteEtHandler {
        async fn handle(&self, _command: DeleteEventType) -> Result<(), ApplicationError> {
            match &self.result {
                Ok(()) => Ok(()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeGetEtHandler {
        result: Result<Option<EventType>, ApplicationError>,
    }

    #[async_trait]
    impl QueryHandler<GetEventTypeById> for FakeGetEtHandler {
        async fn handle(
            &self,
            _query: GetEventTypeById,
        ) -> Result<Option<EventType>, ApplicationError> {
            match &self.result {
                Ok(et) => Ok(et.clone()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeListEtHandler {
        result: Result<PaginatedResult<EventType>, ApplicationError>,
    }

    #[async_trait]
    impl QueryHandler<ListEventTypesByApp> for FakeListEtHandler {
        async fn handle(
            &self,
            _query: ListEventTypesByApp,
        ) -> Result<PaginatedResult<EventType>, ApplicationError> {
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(ApplicationError::Internal(e.to_string())),
            }
        }
    }

    use pigeon_application::ports::stores::ApplicationReadStore;
    use pigeon_domain::application::ApplicationState;

    // --- Helpers ---

    /// Default org_id used in tests — must match the x-org-id header sent in requests.
    fn test_org_id() -> pigeon_domain::organization::OrganizationId {
        // Use a fixed UUID so tests can match it easily
        pigeon_domain::organization::OrganizationId::from_uuid(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        )
    }

    fn fake_event_type() -> EventType {
        EventType::reconstitute(EventTypeState::fake())
    }

    /// Create a fake Application whose app_id and org_id match the test fixtures.
    fn fake_app_for(app_id: &pigeon_domain::application::ApplicationId) -> Application {
        Application::reconstitute(ApplicationState {
            id: app_id.clone(),
            org_id: test_org_id(),
            name: "test-app".to_string(),
            uid: format!("app_{}", Uuid::new_v4()),
            created_at: chrono::Utc::now(),
            version: pigeon_domain::version::Version::new(0),
        })
    }

    fn build_et_state(
        create: FakeCreateEtHandler,
        update: FakeUpdateEtHandler,
        delete: FakeDeleteEtHandler,
        get: FakeGetEtHandler,
        list: FakeListEtHandler,
        app_read_store: Arc<dyn ApplicationReadStore>,
    ) -> AppState {
        use crate::test_support::*;
        AppState {
            create_application: Arc::new(StubCreateAppHandler),
            update_application: Arc::new(StubUpdateAppHandler),
            delete_application: Arc::new(StubDeleteAppHandler),
            send_message: Arc::new(StubSendMessageHandler),
            get_application: Arc::new(StubGetAppHandler),
            list_applications: Arc::new(StubListAppsHandler),
            create_event_type: Arc::new(create),
            update_event_type: Arc::new(update),
            delete_event_type: Arc::new(delete),
            get_event_type: Arc::new(get),
            list_event_types: Arc::new(list),
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
            app_read_store,
            jwks_provider: Arc::new(StubJwksProvider),
            replay_dead_letter: Arc::new(StubReplayDeadLetterHandler),
            retry_attempt: Arc::new(StubRetryAttemptHandler),
            retrigger_message: Arc::new(StubRetriggerMessageHandler),
            send_test_event: Arc::new(StubSendTestEventHandler),
            metrics_render: Arc::new(|| String::new()),
            admin_org_id: None,
        }
    }

    fn default_et_state(et: EventType, app_id: &pigeon_domain::application::ApplicationId) -> AppState {
        use crate::test_support::FakeApplicationReadStore;
        build_et_state(
            FakeCreateEtHandler { result: Ok(et.clone()) },
            FakeUpdateEtHandler { result: Ok(et.clone()) },
            FakeDeleteEtHandler { result: Ok(()) },
            FakeGetEtHandler { result: Ok(Some(et.clone())) },
            FakeListEtHandler {
                result: Ok(PaginatedResult {
                    items: vec![et],
                    total: 1,
                    offset: 0,
                    limit: 20,
                }),
            },
            Arc::new(FakeApplicationReadStore { app: Some(fake_app_for(app_id)) }),
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

    fn org_header() -> &'static str {
        "00000000-0000-0000-0000-000000000001"
    }

    #[tokio::test]
    async fn create_returns_201() {
        let et = fake_event_type();
        let app_id = et.app_id().clone();
        let state = default_et_state(et, &app_id);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/applications/{}/event-types", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", org_header())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "user.created"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let json = body_json(response.into_body()).await;
        assert_eq!(json["name"], "user.created");
    }

    #[tokio::test]
    async fn create_with_empty_name_returns_400() {
        let et = fake_event_type();
        let app_id = et.app_id().clone();
        let state = default_et_state(et, &app_id);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/applications/{}/event-types", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", org_header())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": ""
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
        let et = fake_event_type();
        let app_id = et.app_id().clone();
        let id = *et.id().as_uuid();
        let state = default_et_state(et, &app_id);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/applications/{}/event-types/{id}", app_id.as_uuid()))
                    .header("x-org-id", org_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response.into_body()).await;
        assert!(json["id"].is_string());
    }

    #[tokio::test]
    async fn get_nonexistent_returns_404() {
        let et = fake_event_type();
        let app_id = pigeon_domain::application::ApplicationId::new();
        let id = Uuid::new_v4();
        let state = build_et_state(
            FakeCreateEtHandler { result: Ok(et.clone()) },
            FakeUpdateEtHandler { result: Ok(et.clone()) },
            FakeDeleteEtHandler { result: Ok(()) },
            FakeGetEtHandler { result: Ok(None) },
            FakeListEtHandler {
                result: Ok(PaginatedResult {
                    items: vec![],
                    total: 0,
                    offset: 0,
                    limit: 20,
                }),
            },
            Arc::new(crate::test_support::FakeApplicationReadStore {
                app: Some(fake_app_for(&app_id)),
            }),
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/applications/{}/event-types/{id}", app_id.as_uuid()))
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
        let et = fake_event_type();
        let app_id = et.app_id().clone();
        let state = default_et_state(et, &app_id);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!(
                        "/api/v1/applications/{}/event-types?offset=0&limit=10",
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
        let et = fake_event_type();
        let app_id = et.app_id().clone();
        let id = *et.id().as_uuid();
        let state = default_et_state(et, &app_id);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(&format!("/api/v1/applications/{}/event-types/{id}", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", org_header())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "user.updated",
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
        let et = fake_event_type();
        let app_id = et.app_id().clone();
        let id = *et.id().as_uuid();
        let state = build_et_state(
            FakeCreateEtHandler { result: Ok(et.clone()) },
            FakeUpdateEtHandler { result: Err(ApplicationError::Conflict) },
            FakeDeleteEtHandler { result: Ok(()) },
            FakeGetEtHandler { result: Ok(Some(et.clone())) },
            FakeListEtHandler {
                result: Ok(PaginatedResult {
                    items: vec![],
                    total: 0,
                    offset: 0,
                    limit: 20,
                }),
            },
            Arc::new(crate::test_support::FakeApplicationReadStore {
                app: Some(fake_app_for(&app_id)),
            }),
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(&format!("/api/v1/applications/{}/event-types/{id}", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", org_header())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "user.updated",
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
        let et = fake_event_type();
        let app_id = et.app_id().clone();
        let id = *et.id().as_uuid();
        let state = default_et_state(et, &app_id);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/api/v1/applications/{}/event-types/{id}", app_id.as_uuid()))
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
        let et = fake_event_type();
        let app_id = pigeon_domain::application::ApplicationId::new();
        let id = Uuid::new_v4();
        let state = build_et_state(
            FakeCreateEtHandler { result: Ok(et.clone()) },
            FakeUpdateEtHandler { result: Ok(et.clone()) },
            FakeDeleteEtHandler { result: Err(ApplicationError::NotFound) },
            FakeGetEtHandler { result: Ok(Some(et.clone())) },
            FakeListEtHandler {
                result: Ok(PaginatedResult {
                    items: vec![],
                    total: 0,
                    offset: 0,
                    limit: 20,
                }),
            },
            Arc::new(crate::test_support::FakeApplicationReadStore {
                app: Some(fake_app_for(&app_id)),
            }),
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/api/v1/applications/{}/event-types/{id}", app_id.as_uuid()))
                    .header("x-org-id", org_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_for_wrong_org_returns_404() {
        let et = fake_event_type();
        let app_id = et.app_id().clone();
        // App belongs to test_org_id(), but we send a different org
        let state = default_et_state(et, &app_id);
        let router = test_router(state);

        let wrong_org = Uuid::new_v4();
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/applications/{}/event-types", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", wrong_org.to_string())
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "user.created"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
