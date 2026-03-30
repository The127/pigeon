use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use pigeon_application::commands::create_oidc_config::CreateOidcConfig;
use pigeon_application::commands::delete_oidc_config::DeleteOidcConfig;
use pigeon_application::queries::get_oidc_config_by_id::GetOidcConfigById;
use pigeon_application::queries::list_oidc_configs_by_org::ListOidcConfigsByOrg;
use pigeon_domain::oidc_config::OidcConfigId;
use pigeon_domain::organization::OrganizationId;

use crate::dto::oidc_config::{CreateOidcConfigRequest, OidcConfigResponse};
use crate::dto::pagination::ListQuery;
use crate::error::{ApiError, ErrorBody};
use crate::extractors::AuthInfo;
use crate::state::AppState;
use pigeon_application::mediator::dispatcher::dispatch;

/// Create a new OIDC config for an organization
#[utoipa::path(
    post,
    path = "/admin/v1/organizations/{org_id}/oidc-configs",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = CreateOidcConfigRequest,
    responses(
        (status = 201, description = "OIDC config created", body = OidcConfigResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
    tag = "oidc-configs"
)]
pub(crate) async fn create_oidc_config(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path(org_id): Path<Uuid>,
    Json(body): Json<CreateOidcConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let command = CreateOidcConfig {
        org_id: OrganizationId::from_uuid(org_id),
        issuer_url: body.issuer_url,
        audience: body.audience,
        jwks_url: body.jwks_url,
    };

    let config = dispatch(&*state.create_oidc_config, command, &auth.user_id, &auth.org_id, &*state.audit_store)
        .await
        .map_err(ApiError)?;
    let response = OidcConfigResponse::from(config);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get an OIDC config by ID
#[utoipa::path(
    get,
    path = "/admin/v1/organizations/{org_id}/oidc-configs/{id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("id" = Uuid, Path, description = "OIDC Config ID"),
    ),
    responses(
        (status = 200, description = "OIDC config found", body = OidcConfigResponse),
        (status = 404, description = "OIDC config not found", body = ErrorBody),
    ),
    tag = "oidc-configs"
)]
// Intentionally unauthenticated — OIDC config is public metadata
// required by the frontend before login can occur.
// Read-only, no sensitive data exposed.
pub(crate) async fn get_oidc_config(
    State(state): State<AppState>,
    Path((_org_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let query = GetOidcConfigById {
        id: OidcConfigId::from_uuid(id),
    };

    let config = state
        .get_oidc_config
        .handle(query)
        .await
        .map_err(ApiError)?
        .ok_or(ApiError(pigeon_application::error::ApplicationError::NotFound))?;

    Ok(Json(OidcConfigResponse::from(config)))
}

/// List OIDC configs for an organization
#[utoipa::path(
    get,
    path = "/admin/v1/organizations/{org_id}/oidc-configs",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ListQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of OIDC configs"),
    ),
    tag = "oidc-configs"
)]
pub(crate) async fn list_oidc_configs(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let list_query = ListOidcConfigsByOrg {
        org_id: OrganizationId::from_uuid(org_id),
        offset: query.offset.unwrap_or(0),
        limit: query.limit.unwrap_or(20),
    };

    let result = state
        .list_oidc_configs
        .handle(list_query)
        .await
        .map_err(ApiError)?;

    let response = serde_json::json!({
        "items": result.items.into_iter().map(OidcConfigResponse::from).collect::<Vec<_>>(),
        "total": result.total,
        "offset": result.offset,
        "limit": result.limit,
    });

    Ok(Json(response))
}

/// Delete an OIDC config
#[utoipa::path(
    delete,
    path = "/admin/v1/organizations/{org_id}/oidc-configs/{id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("id" = Uuid, Path, description = "OIDC Config ID"),
    ),
    responses(
        (status = 204, description = "OIDC config deleted"),
        (status = 404, description = "OIDC config not found", body = ErrorBody),
    ),
    tag = "oidc-configs"
)]
pub(crate) async fn delete_oidc_config(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path((_org_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let command = DeleteOidcConfig {
        id: OidcConfigId::from_uuid(id),
    };

    dispatch(&*state.delete_oidc_config, command, &auth.user_id, &auth.org_id, &*state.audit_store)
        .await
        .map_err(ApiError)?;

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
    use pigeon_application::commands::create_event_type::CreateEventType;
    use pigeon_application::commands::delete_application::DeleteApplication;
    use pigeon_application::commands::delete_endpoint::DeleteEndpoint;
    use pigeon_application::commands::delete_event_type::DeleteEventType;
    use pigeon_application::commands::delete_organization::DeleteOrganization;
    use pigeon_application::commands::send_message::{SendMessage, SendMessageResult};
    use pigeon_application::commands::update_application::UpdateApplication;
    use pigeon_application::commands::update_endpoint::UpdateEndpoint;
    use pigeon_application::commands::update_event_type::UpdateEventType;
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
    use pigeon_domain::application::Application;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::oidc_config::{OidcConfig, OidcConfigState};
    use pigeon_domain::organization::Organization;
    use std::sync::Arc;
    use tower::ServiceExt;

    // Stubs for non-oidc handlers
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
        async fn handle(&self, _c: SendMessage) -> Result<SendMessageResult, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }
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
        async fn check(&self) -> bool { true }
    }
    struct StubCreateOrgHandler;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::create_organization::CreateOrganization> for StubCreateOrgHandler {
        async fn handle(&self, _c: pigeon_application::commands::create_organization::CreateOrganization) -> Result<Organization, ApplicationError> {
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

    // --- OIDC config fakes ---

    struct FakeCreateOidcConfigHandler {
        result: Result<OidcConfig, ApplicationError>,
    }
    #[async_trait]
    impl CommandHandler<CreateOidcConfig> for FakeCreateOidcConfigHandler {
        async fn handle(&self, command: CreateOidcConfig) -> Result<OidcConfig, ApplicationError> {
            match &self.result {
                Ok(_) => OidcConfig::new(
                    command.org_id,
                    command.issuer_url,
                    command.audience,
                    command.jwks_url,
                )
                .map_err(|e| ApplicationError::Validation(e.to_string())),
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeDeleteOidcConfigHandler {
        result: Result<(), ApplicationError>,
    }
    #[async_trait]
    impl CommandHandler<DeleteOidcConfig> for FakeDeleteOidcConfigHandler {
        async fn handle(&self, _command: DeleteOidcConfig) -> Result<(), ApplicationError> {
            match &self.result {
                Ok(()) => Ok(()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeGetOidcConfigHandler {
        result: Result<Option<OidcConfig>, ApplicationError>,
    }
    #[async_trait]
    impl QueryHandler<GetOidcConfigById> for FakeGetOidcConfigHandler {
        async fn handle(&self, _query: GetOidcConfigById) -> Result<Option<OidcConfig>, ApplicationError> {
            match &self.result {
                Ok(c) => Ok(c.clone()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeListOidcConfigsHandler {
        result: Result<PaginatedResult<OidcConfig>, ApplicationError>,
    }
    #[async_trait]
    impl QueryHandler<ListOidcConfigsByOrg> for FakeListOidcConfigsHandler {
        async fn handle(&self, _query: ListOidcConfigsByOrg) -> Result<PaginatedResult<OidcConfig>, ApplicationError> {
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(ApplicationError::Internal(e.to_string())),
            }
        }
    }

    // --- Helpers ---

    fn fake_oidc_config() -> OidcConfig {
        OidcConfig::reconstitute(OidcConfigState::fake())
    }

    fn build_state(
        create: FakeCreateOidcConfigHandler,
        delete: FakeDeleteOidcConfigHandler,
        get: FakeGetOidcConfigHandler,
        list: FakeListOidcConfigsHandler,
    ) -> crate::state::AppState {
        use crate::test_support::*;
        crate::state::AppState {
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
            create_oidc_config: Arc::new(create),
            delete_oidc_config: Arc::new(delete),
            get_oidc_config: Arc::new(get),
            list_oidc_configs: Arc::new(list),
            oidc_config_read_store: Arc::new(StubOidcConfigReadStore),
            org_read_store: Arc::new(StubOrganizationReadStore),
            jwks_provider: Arc::new(StubJwksProvider),
            replay_dead_letter: Arc::new(StubReplayDeadLetterHandler),
            retry_attempt: Arc::new(StubRetryAttemptHandler),
            retrigger_message: Arc::new(StubRetriggerMessageHandler),
            send_test_event: Arc::new(StubSendTestEventHandler),
            list_audit_log: Arc::new(StubListAuditLogHandler),
            audit_store: Arc::new(StubAuditStore),
            metrics_render: Arc::new(|| String::new()),
            admin_org_id: None,
        }
    }

    fn default_state_with_config(config: OidcConfig) -> crate::state::AppState {
        build_state(
            FakeCreateOidcConfigHandler { result: Ok(config.clone()) },
            FakeDeleteOidcConfigHandler { result: Ok(()) },
            FakeGetOidcConfigHandler { result: Ok(Some(config.clone())) },
            FakeListOidcConfigsHandler {
                result: Ok(PaginatedResult {
                    items: vec![config],
                    total: 1,
                    offset: 0,
                    limit: 20,
                }),
            },
        )
    }

    fn test_router(state: crate::state::AppState) -> Router {
        crate::router_without_auth(state)
    }

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    const TEST_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

    // --- Tests ---

    #[tokio::test]
    async fn create_returns_201() {
        let config = fake_oidc_config();
        let state = default_state_with_config(config);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/admin/v1/organizations/{TEST_ORG_ID}/oidc-configs"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "issuer_url": "https://auth.example.com",
                            "audience": "my-api",
                            "jwks_url": "https://auth.example.com/.well-known/jwks.json"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let json = body_json(response.into_body()).await;
        assert_eq!(json["issuer_url"], "https://auth.example.com");
        assert_eq!(json["audience"], "my-api");
    }

    #[tokio::test]
    async fn create_with_empty_issuer_returns_400() {
        let config = fake_oidc_config();
        let state = default_state_with_config(config);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/admin/v1/organizations/{TEST_ORG_ID}/oidc-configs"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "issuer_url": "",
                            "audience": "my-api",
                            "jwks_url": "https://auth.example.com/.well-known/jwks.json"
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
        let config = fake_oidc_config();
        let config_id = *config.id().as_uuid();
        let state = default_state_with_config(config);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!(
                        "/admin/v1/organizations/{TEST_ORG_ID}/oidc-configs/{config_id}"
                    ))
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
        let config = fake_oidc_config();
        let state = build_state(
            FakeCreateOidcConfigHandler { result: Ok(config) },
            FakeDeleteOidcConfigHandler { result: Ok(()) },
            FakeGetOidcConfigHandler { result: Ok(None) },
            FakeListOidcConfigsHandler {
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
                    .uri(&format!(
                        "/admin/v1/organizations/{TEST_ORG_ID}/oidc-configs/{id}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_returns_paginated() {
        let config = fake_oidc_config();
        let state = default_state_with_config(config);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!(
                        "/admin/v1/organizations/{TEST_ORG_ID}/oidc-configs?offset=0&limit=10"
                    ))
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
    async fn delete_returns_204() {
        let config = fake_oidc_config();
        let config_id = *config.id().as_uuid();
        let state = default_state_with_config(config);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!(
                        "/admin/v1/organizations/{TEST_ORG_ID}/oidc-configs/{config_id}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_404() {
        let config = fake_oidc_config();
        let state = build_state(
            FakeCreateOidcConfigHandler { result: Ok(config) },
            FakeDeleteOidcConfigHandler { result: Err(ApplicationError::NotFound) },
            FakeGetOidcConfigHandler { result: Ok(None) },
            FakeListOidcConfigsHandler {
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
                    .uri(&format!(
                        "/admin/v1/organizations/{TEST_ORG_ID}/oidc-configs/{id}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
