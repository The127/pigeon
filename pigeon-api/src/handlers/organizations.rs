use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use pigeon_application::commands::create_organization::CreateOrganization;
use pigeon_application::commands::delete_organization::DeleteOrganization;
use pigeon_application::commands::update_organization::UpdateOrganization;
use pigeon_application::queries::get_organization_by_id::GetOrganizationById;
use pigeon_application::queries::list_organizations::ListOrganizations;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;

use crate::dto::organization::{
    CreateOrganizationRequest, OrganizationResponse, UpdateOrganizationRequest,
};
use crate::dto::pagination::ListQuery;
use crate::error::{ApiError, ErrorBody};
use crate::state::AppState;

/// Create a new organization
#[utoipa::path(
    post,
    path = "/admin/v1/organizations",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created", body = OrganizationResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
    tag = "organizations"
)]
pub async fn create_organization(
    State(state): State<AppState>,
    Json(body): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let command = CreateOrganization {
        name: body.name,
        slug: body.slug,
        oidc_issuer_url: body.oidc_issuer_url,
        oidc_audience: body.oidc_audience,
        oidc_jwks_url: body.oidc_jwks_url,
    };

    let org = state
        .create_organization
        .handle(command)
        .await
        .map_err(ApiError)?;
    let response = OrganizationResponse::from(org);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get an organization by ID
#[utoipa::path(
    get,
    path = "/admin/v1/organizations/{id}",
    params(("id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "Organization found", body = OrganizationResponse),
        (status = 404, description = "Organization not found", body = ErrorBody),
    ),
    tag = "organizations"
)]
pub async fn get_organization(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let query = GetOrganizationById {
        id: OrganizationId::from_uuid(id),
    };

    let org = state
        .get_organization
        .handle(query)
        .await
        .map_err(ApiError)?
        .ok_or(ApiError(pigeon_application::error::ApplicationError::NotFound))?;

    Ok(Json(OrganizationResponse::from(org)))
}

/// List organizations with pagination
#[utoipa::path(
    get,
    path = "/admin/v1/organizations",
    params(ListQuery),
    responses(
        (status = 200, description = "Paginated list of organizations"),
    ),
    tag = "organizations"
)]
pub async fn list_organizations(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let list_query = ListOrganizations {
        offset: query.offset.unwrap_or(0),
        limit: query.limit.unwrap_or(20),
    };

    let result = state
        .list_organizations
        .handle(list_query)
        .await
        .map_err(ApiError)?;

    let response = serde_json::json!({
        "items": result.items.into_iter().map(OrganizationResponse::from).collect::<Vec<_>>(),
        "total": result.total,
        "offset": result.offset,
        "limit": result.limit,
    });

    Ok(Json(response))
}

/// Update an organization
#[utoipa::path(
    put,
    path = "/admin/v1/organizations/{id}",
    params(("id" = Uuid, Path, description = "Organization ID")),
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated", body = OrganizationResponse),
        (status = 404, description = "Organization not found", body = ErrorBody),
        (status = 409, description = "Version conflict", body = ErrorBody),
    ),
    tag = "organizations"
)]
pub async fn update_organization(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateOrganizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let command = UpdateOrganization {
        id: OrganizationId::from_uuid(id),
        name: body.name,
        version: Version::new(body.version),
    };

    let org = state
        .update_organization
        .handle(command)
        .await
        .map_err(ApiError)?;

    Ok(Json(OrganizationResponse::from(org)))
}

/// Delete an organization
#[utoipa::path(
    delete,
    path = "/admin/v1/organizations/{id}",
    params(("id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 204, description = "Organization deleted"),
        (status = 404, description = "Organization not found", body = ErrorBody),
    ),
    tag = "organizations"
)]
pub async fn delete_organization(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let command = DeleteOrganization {
        id: OrganizationId::from_uuid(id),
    };

    state
        .delete_organization
        .handle(command)
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
    use pigeon_application::commands::send_message::{SendMessage, SendMessageResult};
    use pigeon_application::commands::update_application::UpdateApplication;
    use pigeon_application::commands::update_endpoint::UpdateEndpoint;
    use pigeon_application::commands::update_event_type::UpdateEventType;
    use pigeon_application::error::ApplicationError;
    use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
    use pigeon_application::queries::get_application_by_id::GetApplicationById;
    use pigeon_application::queries::get_endpoint_by_id::GetEndpointById;
    use pigeon_application::queries::get_event_type_by_id::GetEventTypeById;
    use pigeon_application::queries::list_applications::ListApplications;
    use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp;
    use pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp;
    use pigeon_application::queries::PaginatedResult;
    use pigeon_domain::application::Application;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::organization::{Organization, OrganizationState};
    use std::sync::Arc;
    use tower::ServiceExt;

    // --- Stubs for all non-organization handlers ---

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
        async fn handle(
            &self,
            _q: GetEndpointById,
        ) -> Result<Option<Endpoint>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListEpHandler;
    #[async_trait]
    impl QueryHandler<ListEndpointsByApp> for StubListEpHandler {
        async fn handle(
            &self,
            _q: ListEndpointsByApp,
        ) -> Result<PaginatedResult<Endpoint>, ApplicationError> {
            Ok(PaginatedResult {
                items: vec![],
                total: 0,
                offset: 0,
                limit: 20,
            })
        }
    }

    struct StubHealthChecker;
    #[async_trait]
    impl pigeon_application::ports::health::HealthChecker for StubHealthChecker {
        async fn check(&self) -> bool {
            true
        }
    }

    // --- Organization fakes ---

    struct FakeCreateOrgHandler {
        result: Result<Organization, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<CreateOrganization> for FakeCreateOrgHandler {
        async fn handle(
            &self,
            command: CreateOrganization,
        ) -> Result<Organization, ApplicationError> {
            match &self.result {
                Ok(_) => Organization::new(command.name, command.slug)
                    .map_err(|e| ApplicationError::Validation(e.to_string())),
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeUpdateOrgHandler {
        result: Result<Organization, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<UpdateOrganization> for FakeUpdateOrgHandler {
        async fn handle(
            &self,
            _command: UpdateOrganization,
        ) -> Result<Organization, ApplicationError> {
            match &self.result {
                Ok(org) => Ok(org.clone()),
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    ApplicationError::Conflict => ApplicationError::Conflict,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeDeleteOrgHandler {
        result: Result<(), ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<DeleteOrganization> for FakeDeleteOrgHandler {
        async fn handle(
            &self,
            _command: DeleteOrganization,
        ) -> Result<(), ApplicationError> {
            match &self.result {
                Ok(()) => Ok(()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeGetOrgHandler {
        result: Result<Option<Organization>, ApplicationError>,
    }

    #[async_trait]
    impl QueryHandler<GetOrganizationById> for FakeGetOrgHandler {
        async fn handle(
            &self,
            _query: GetOrganizationById,
        ) -> Result<Option<Organization>, ApplicationError> {
            match &self.result {
                Ok(org) => Ok(org.clone()),
                Err(e) => Err(match e {
                    ApplicationError::NotFound => ApplicationError::NotFound,
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    struct FakeListOrgHandler {
        result: Result<PaginatedResult<Organization>, ApplicationError>,
    }

    #[async_trait]
    impl QueryHandler<ListOrganizations> for FakeListOrgHandler {
        async fn handle(
            &self,
            _query: ListOrganizations,
        ) -> Result<PaginatedResult<Organization>, ApplicationError> {
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(ApplicationError::Internal(e.to_string())),
            }
        }
    }

    // --- Helpers ---

    fn fake_org() -> Organization {
        Organization::reconstitute(OrganizationState::fake())
    }

    fn build_state(
        create: FakeCreateOrgHandler,
        update: FakeUpdateOrgHandler,
        delete: FakeDeleteOrgHandler,
        get: FakeGetOrgHandler,
        list: FakeListOrgHandler,
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
            create_endpoint: Arc::new(StubCreateEpHandler),
            update_endpoint: Arc::new(StubUpdateEpHandler),
            delete_endpoint: Arc::new(StubDeleteEpHandler),
            get_endpoint: Arc::new(StubGetEpHandler),
            list_endpoints: Arc::new(StubListEpHandler),
            get_message: Arc::new(StubGetMessageHandler),
            list_messages: Arc::new(StubListMessagesHandler),
            list_attempts: Arc::new(StubListAttemptsHandler),
            get_dead_letter: Arc::new(StubGetDeadLetterHandler),
            list_dead_letters: Arc::new(StubListDeadLettersHandler),
            health_checker: Arc::new(StubHealthChecker),
            create_organization: Arc::new(create),
            update_organization: Arc::new(update),
            delete_organization: Arc::new(delete),
            get_organization: Arc::new(get),
            list_organizations: Arc::new(list),
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
            send_test_event: Arc::new(StubSendTestEventHandler),
            metrics_render: Arc::new(|| String::new()),
            admin_org_id: None,
        }
    }

    fn default_state_with_org(org: Organization) -> AppState {
        build_state(
            FakeCreateOrgHandler {
                result: Ok(org.clone()),
            },
            FakeUpdateOrgHandler {
                result: Ok(org.clone()),
            },
            FakeDeleteOrgHandler { result: Ok(()) },
            FakeGetOrgHandler {
                result: Ok(Some(org.clone())),
            },
            FakeListOrgHandler {
                result: Ok(PaginatedResult {
                    items: vec![org],
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
        let org = fake_org();
        let state = default_state_with_org(org);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/v1/organizations")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "my-org",
                            "slug": "my-org",
                            "oidc_issuer_url": "https://auth.example.com",
                            "oidc_audience": "pigeon-api",
                            "oidc_jwks_url": "https://auth.example.com/.well-known/jwks.json"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let json = body_json(response.into_body()).await;
        assert_eq!(json["name"], "my-org");
        assert_eq!(json["slug"], "my-org");
    }

    #[tokio::test]
    async fn create_with_empty_name_returns_400() {
        let org = fake_org();
        let state = default_state_with_org(org);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/v1/organizations")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "name": "",
                            "slug": "my-org",
                            "oidc_issuer_url": "https://auth.example.com",
                            "oidc_audience": "pigeon-api",
                            "oidc_jwks_url": "https://auth.example.com/.well-known/jwks.json"
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
        let org = fake_org();
        let id = *org.id().as_uuid();
        let state = default_state_with_org(org);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/admin/v1/organizations/{id}"))
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
        let org = fake_org();
        let state = build_state(
            FakeCreateOrgHandler {
                result: Ok(org.clone()),
            },
            FakeUpdateOrgHandler {
                result: Ok(org.clone()),
            },
            FakeDeleteOrgHandler { result: Ok(()) },
            FakeGetOrgHandler { result: Ok(None) },
            FakeListOrgHandler {
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
                    .uri(&format!("/admin/v1/organizations/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_returns_paginated() {
        let org = fake_org();
        let state = default_state_with_org(org);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/admin/v1/organizations?offset=0&limit=10")
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
        let org = fake_org();
        let id = *org.id().as_uuid();
        let state = default_state_with_org(org);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(&format!("/admin/v1/organizations/{id}"))
                    .header("content-type", "application/json")
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
        let org = fake_org();
        let id = *org.id().as_uuid();
        let state = build_state(
            FakeCreateOrgHandler {
                result: Ok(org.clone()),
            },
            FakeUpdateOrgHandler {
                result: Err(ApplicationError::Conflict),
            },
            FakeDeleteOrgHandler { result: Ok(()) },
            FakeGetOrgHandler {
                result: Ok(Some(org.clone())),
            },
            FakeListOrgHandler {
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
                    .uri(&format!("/admin/v1/organizations/{id}"))
                    .header("content-type", "application/json")
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
        let org = fake_org();
        let id = *org.id().as_uuid();
        let state = default_state_with_org(org);
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/admin/v1/organizations/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_404() {
        let org = fake_org();
        let state = build_state(
            FakeCreateOrgHandler {
                result: Ok(org.clone()),
            },
            FakeUpdateOrgHandler {
                result: Ok(org.clone()),
            },
            FakeDeleteOrgHandler {
                result: Err(ApplicationError::NotFound),
            },
            FakeGetOrgHandler {
                result: Ok(Some(org.clone())),
            },
            FakeListOrgHandler {
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
                    .uri(&format!("/admin/v1/organizations/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
