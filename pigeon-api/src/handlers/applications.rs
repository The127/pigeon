use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use pigeon_application::commands::create_application::CreateApplication;
use pigeon_application::commands::delete_application::DeleteApplication;
use pigeon_application::commands::update_application::UpdateApplication;
use pigeon_application::queries::get_application_by_id::GetApplicationById;
use pigeon_application::queries::list_applications::ListApplications;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::version::Version;

use crate::dto::application::{ApplicationResponse, CreateApplicationRequest, UpdateApplicationRequest};
use crate::dto::pagination::{ListQuery, PaginatedResponse};
use crate::error::{ApiError, ErrorBody};
use crate::extractors::OrgId;
use crate::state::AppState;

/// Create a new application
#[utoipa::path(
    post,
    path = "/api/v1/applications",
    request_body = CreateApplicationRequest,
    responses(
        (status = 201, description = "Application created", body = ApplicationResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
    tag = "applications"
)]
pub async fn create_application(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Json(body): Json<CreateApplicationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let command = CreateApplication {
        org_id,
        name: body.name,
        uid: body.uid,
    };

    let app = state.create_application.handle(command).await.map_err(ApiError)?;
    let response = ApplicationResponse::from(app);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get an application by ID
#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}",
    params(("id" = Uuid, Path, description = "Application ID")),
    responses(
        (status = 200, description = "Application found", body = ApplicationResponse),
        (status = 404, description = "Application not found", body = ErrorBody),
    ),
    tag = "applications"
)]
pub async fn get_application(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let query = GetApplicationById {
        org_id,
        id: ApplicationId::from_uuid(id),
    };

    let app = state
        .get_application
        .handle(query)
        .await
        .map_err(ApiError)?
        .ok_or(ApiError(pigeon_application::error::ApplicationError::NotFound))?;

    Ok(Json(ApplicationResponse::from(app)))
}

/// List applications with pagination
#[utoipa::path(
    get,
    path = "/api/v1/applications",
    params(ListQuery),
    responses(
        (status = 200, description = "Paginated list of applications", body = PaginatedResponse),
    ),
    tag = "applications"
)]
pub async fn list_applications(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let list_query = ListApplications {
        org_id,
        offset: query.offset.unwrap_or(0),
        limit: query.limit.unwrap_or(20),
    };

    let result = state.list_applications.handle(list_query).await.map_err(ApiError)?;

    let response = PaginatedResponse {
        items: result.items.into_iter().map(ApplicationResponse::from).collect(),
        total: result.total,
        offset: result.offset,
        limit: result.limit,
    };

    Ok(Json(response))
}

/// Update an application
#[utoipa::path(
    put,
    path = "/api/v1/applications/{id}",
    params(("id" = Uuid, Path, description = "Application ID")),
    request_body = UpdateApplicationRequest,
    responses(
        (status = 200, description = "Application updated", body = ApplicationResponse),
        (status = 404, description = "Application not found", body = ErrorBody),
        (status = 409, description = "Version conflict", body = ErrorBody),
    ),
    tag = "applications"
)]
pub async fn update_application(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateApplicationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let command = UpdateApplication {
        org_id,
        id: ApplicationId::from_uuid(id),
        name: body.name,
        version: Version::new(body.version),
    };

    let app = state.update_application.handle(command).await.map_err(ApiError)?;

    Ok(Json(ApplicationResponse::from(app)))
}

/// Delete an application
#[utoipa::path(
    delete,
    path = "/api/v1/applications/{id}",
    params(("id" = Uuid, Path, description = "Application ID")),
    responses(
        (status = 204, description = "Application deleted"),
        (status = 404, description = "Application not found", body = ErrorBody),
    ),
    tag = "applications"
)]
pub async fn delete_application(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let command = DeleteApplication {
        org_id,
        id: ApplicationId::from_uuid(id),
    };

    state.delete_application.handle(command).await.map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;
    use pigeon_application::commands::create_endpoint::CreateEndpoint;
    use pigeon_application::commands::create_event_type::CreateEventType;
    use pigeon_application::commands::delete_endpoint::DeleteEndpoint;
    use pigeon_application::commands::delete_event_type::DeleteEventType;
    use pigeon_application::commands::send_message::{SendMessage, SendMessageResult};
    use pigeon_application::commands::update_endpoint::UpdateEndpoint;
    use pigeon_application::commands::update_event_type::UpdateEventType;
    use pigeon_application::error::ApplicationError;
    use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
    use pigeon_application::queries::get_endpoint_by_id::GetEndpointById;
    use pigeon_application::queries::get_event_type_by_id::GetEventTypeById;
    use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp;
    use pigeon_application::commands::create_organization::CreateOrganization;
    use pigeon_application::commands::delete_organization::DeleteOrganization;
    use pigeon_application::commands::update_organization::UpdateOrganization;
    use pigeon_application::queries::get_organization_by_id::GetOrganizationById;
    use pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp;
    use pigeon_application::queries::list_organizations::ListOrganizations;
    use pigeon_application::queries::PaginatedResult;
    use pigeon_domain::application::{Application, ApplicationState as DomainAppState};
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::organization::Organization;
    use std::sync::Arc;
    use tower::ServiceExt;

    // --- Fakes ---

    struct FakeCreateHandler {
        result: Result<Application, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<CreateApplication> for FakeCreateHandler {
        async fn handle(&self, command: CreateApplication) -> Result<Application, ApplicationError> {
            match &self.result {
                Ok(_) => {
                    Application::new(command.org_id, command.name, command.uid)
                        .map_err(|e| ApplicationError::Validation(e.to_string()))
                }
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    ApplicationError::Conflict => ApplicationError::Conflict,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeUpdateHandler {
        result: Result<Application, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<UpdateApplication> for FakeUpdateHandler {
        async fn handle(&self, _command: UpdateApplication) -> Result<Application, ApplicationError> {
            match &self.result {
                Ok(app) => Ok(app.clone()),
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    ApplicationError::Conflict => ApplicationError::Conflict,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeDeleteHandler {
        result: Result<(), ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<DeleteApplication> for FakeDeleteHandler {
        async fn handle(&self, _command: DeleteApplication) -> Result<(), ApplicationError> {
            match &self.result {
                Ok(()) => Ok(()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeGetHandler {
        result: Result<Option<Application>, ApplicationError>,
    }

    #[async_trait]
    impl QueryHandler<GetApplicationById> for FakeGetHandler {
        async fn handle(
            &self,
            _query: GetApplicationById,
        ) -> Result<Option<Application>, ApplicationError> {
            match &self.result {
                Ok(app) => Ok(app.clone()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeListHandler {
        result: Result<PaginatedResult<Application>, ApplicationError>,
    }

    #[async_trait]
    impl QueryHandler<ListApplications> for FakeListHandler {
        async fn handle(
            &self,
            _query: ListApplications,
        ) -> Result<PaginatedResult<Application>, ApplicationError> {
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(match e {
                    _ => ApplicationError::Internal(e.to_string()),
                }),
            }
        }
    }

    // --- Send message stub ---

    struct StubSendMessageHandler;
    #[async_trait]
    impl CommandHandler<SendMessage> for StubSendMessageHandler {
        async fn handle(&self, _c: SendMessage) -> Result<SendMessageResult, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    // --- Event type stubs ---

    struct StubCreateEtHandler;
    #[async_trait]
    impl CommandHandler<CreateEventType> for StubCreateEtHandler {
        async fn handle(&self, _c: CreateEventType) -> Result<EventType, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateEtHandler;
    #[async_trait]
    impl CommandHandler<UpdateEventType> for StubUpdateEtHandler {
        async fn handle(&self, _c: UpdateEventType) -> Result<EventType, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteEtHandler;
    #[async_trait]
    impl CommandHandler<DeleteEventType> for StubDeleteEtHandler {
        async fn handle(&self, _c: DeleteEventType) -> Result<(), ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubGetEtHandler;
    #[async_trait]
    impl QueryHandler<GetEventTypeById> for StubGetEtHandler {
        async fn handle(
            &self,
            _q: GetEventTypeById,
        ) -> Result<Option<EventType>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListEtHandler;
    #[async_trait]
    impl QueryHandler<ListEventTypesByApp> for StubListEtHandler {
        async fn handle(
            &self,
            _q: ListEventTypesByApp,
        ) -> Result<PaginatedResult<EventType>, ApplicationError> {
            Ok(PaginatedResult {
                items: vec![],
                total: 0,
                offset: 0,
                limit: 20,
            })
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

    // --- Health fake ---

    struct FakeHealthChecker;
    #[async_trait]
    impl pigeon_application::ports::health::HealthChecker for FakeHealthChecker {
        async fn check(&self) -> bool {
            true
        }
    }

    // --- Organization stubs ---

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

    // --- Helpers ---

    fn fake_app() -> Application {
        Application::reconstitute(DomainAppState::fake())
    }

    fn build_state(
        create: FakeCreateHandler,
        update: FakeUpdateHandler,
        delete: FakeDeleteHandler,
        get: FakeGetHandler,
        list: FakeListHandler,
    ) -> AppState {
        use crate::test_support::*;
        AppState {
            create_application: Arc::new(create),
            update_application: Arc::new(update),
            delete_application: Arc::new(delete),
            send_message: Arc::new(StubSendMessageHandler),
            get_application: Arc::new(get),
            list_applications: Arc::new(list),
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
            get_message: Arc::new(StubGetMessageHandler),
            list_messages: Arc::new(StubListMessagesHandler),
            list_attempts: Arc::new(StubListAttemptsHandler),
            get_dead_letter: Arc::new(StubGetDeadLetterHandler),
            list_dead_letters: Arc::new(StubListDeadLettersHandler),
            health_checker: Arc::new(FakeHealthChecker),
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
            app_read_store: Arc::new(StubApplicationReadStore),
            jwks_provider: Arc::new(StubJwksProvider),
            replay_dead_letter: Arc::new(StubReplayDeadLetterHandler),
            retry_attempt: Arc::new(StubRetryAttemptHandler),
            retrigger_message: Arc::new(StubRetriggerMessageHandler),
            send_test_event: Arc::new(StubSendTestEventHandler),
            metrics_render: Arc::new(|| String::new()),
            admin_org_id: None,
        }
    }

    fn default_state_with_app(app: Application) -> AppState {
        build_state(
            FakeCreateHandler { result: Ok(app.clone()) },
            FakeUpdateHandler { result: Ok(app.clone()) },
            FakeDeleteHandler { result: Ok(()) },
            FakeGetHandler { result: Ok(Some(app.clone())) },
            FakeListHandler {
                result: Ok(PaginatedResult {
                    items: vec![app],
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

    const TEST_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

    #[tokio::test]
    async fn create_returns_201() {
        let app = fake_app();
        let state = default_state_with_app(app);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/applications")
                    .header("content-type", "application/json")
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "my-app",
                            "uid": "app_123"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let json = body_json(response.into_body()).await;
        assert_eq!(json["name"], "my-app");
        assert_eq!(json["uid"], "app_123");
    }

    #[tokio::test]
    async fn create_without_org_id_returns_401() {
        let app = fake_app();
        let state = default_state_with_app(app);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/applications")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "my-app",
                            "uid": "app_123"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn create_with_empty_name_returns_400() {
        let app = fake_app();
        let state = default_state_with_app(app);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/applications")
                    .header("content-type", "application/json")
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "",
                            "uid": "app_123"
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
        let app = fake_app();
        let id = *app.id().as_uuid();
        let state = default_state_with_app(app);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/applications/{id}"))
                    .header("x-org-id", TEST_ORG_ID)
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
    async fn get_without_org_id_returns_401() {
        let app = fake_app();
        let id = *app.id().as_uuid();
        let state = default_state_with_app(app);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/applications/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn get_nonexistent_returns_404() {
        let app = fake_app();
        let state = build_state(
            FakeCreateHandler { result: Ok(app.clone()) },
            FakeUpdateHandler { result: Ok(app.clone()) },
            FakeDeleteHandler { result: Ok(()) },
            FakeGetHandler { result: Ok(None) },
            FakeListHandler {
                result: Ok(PaginatedResult {
                    items: vec![],
                    total: 0,
                    offset: 0,
                    limit: 20,
                }),
            },
        );
        let router = test_router(state);

        let id = Uuid::new_v4();
        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/applications/{id}"))
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_returns_paginated() {
        let app = fake_app();
        let state = default_state_with_app(app);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/applications?offset=0&limit=10")
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response.into_body()).await;
        assert!(json["items"].is_array());
        assert_eq!(json["total"], 1);
        assert_eq!(json["offset"], 0);
        assert_eq!(json["limit"], 20);
    }

    #[tokio::test]
    async fn update_returns_200() {
        let app = fake_app();
        let id = *app.id().as_uuid();
        let state = default_state_with_app(app);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(&format!("/api/v1/applications/{id}"))
                    .header("content-type", "application/json")
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "updated-name",
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
        let app = fake_app();
        let id = *app.id().as_uuid();
        let state = build_state(
            FakeCreateHandler { result: Ok(app.clone()) },
            FakeUpdateHandler { result: Err(ApplicationError::Conflict) },
            FakeDeleteHandler { result: Ok(()) },
            FakeGetHandler { result: Ok(Some(app.clone())) },
            FakeListHandler {
                result: Ok(PaginatedResult {
                    items: vec![],
                    total: 0,
                    offset: 0,
                    limit: 20,
                }),
            },
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(&format!("/api/v1/applications/{id}"))
                    .header("content-type", "application/json")
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "updated-name",
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
        let app = fake_app();
        let id = *app.id().as_uuid();
        let state = default_state_with_app(app);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/api/v1/applications/{id}"))
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_404() {
        let app = fake_app();
        let state = build_state(
            FakeCreateHandler { result: Ok(app.clone()) },
            FakeUpdateHandler { result: Ok(app.clone()) },
            FakeDeleteHandler { result: Err(ApplicationError::NotFound) },
            FakeGetHandler { result: Ok(Some(app.clone())) },
            FakeListHandler {
                result: Ok(PaginatedResult {
                    items: vec![],
                    total: 0,
                    offset: 0,
                    limit: 20,
                }),
            },
        );
        let router = test_router(state);

        let id = Uuid::new_v4();
        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/api/v1/applications/{id}"))
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
