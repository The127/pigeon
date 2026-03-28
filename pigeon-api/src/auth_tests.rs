//! Tests for the production auth middleware's rejection paths.
//!
//! These use the PRODUCTION `router()` (with real JWT auth middleware), not
//! `router_without_auth`. The StubOidcConfigReadStore returns `None` for all
//! lookups, so every token fails the config lookup step. This lets us verify
//! the rejection paths without needing real JWTs.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use pigeon_application::commands::create_application::CreateApplication;
use pigeon_application::commands::create_endpoint::CreateEndpoint;
use pigeon_application::commands::create_event_type::CreateEventType;
use pigeon_application::commands::create_oidc_config::CreateOidcConfig;
use pigeon_application::commands::delete_application::DeleteApplication;
use pigeon_application::commands::delete_endpoint::DeleteEndpoint;
use pigeon_application::commands::delete_event_type::DeleteEventType;
use pigeon_application::commands::delete_oidc_config::DeleteOidcConfig;
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
use pigeon_application::queries::get_oidc_config_by_id::GetOidcConfigById;
use pigeon_application::queries::get_organization_by_id::GetOrganizationById;
use pigeon_application::queries::list_applications::ListApplications;
use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp;
use pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp;
use pigeon_application::queries::list_oidc_configs_by_org::ListOidcConfigsByOrg;
use pigeon_application::queries::list_organizations::ListOrganizations;
use pigeon_application::queries::PaginatedResult;
use pigeon_domain::application::Application;
use pigeon_domain::endpoint::Endpoint;
use pigeon_domain::event_type::EventType;
use pigeon_domain::oidc_config::OidcConfig;
use pigeon_domain::organization::Organization;
use tower::ServiceExt;

use crate::state::AppState;
use crate::test_support::*;

// Stubs -- these are never actually called because the auth middleware rejects first.
struct S;

#[async_trait]
impl CommandHandler<CreateApplication> for S {
    async fn handle(&self, _: CreateApplication) -> Result<Application, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<UpdateApplication> for S {
    async fn handle(&self, _: UpdateApplication) -> Result<Application, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<DeleteApplication> for S {
    async fn handle(&self, _: DeleteApplication) -> Result<(), ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<SendMessage> for S {
    async fn handle(&self, _: SendMessage) -> Result<SendMessageResult, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<pigeon_application::commands::replay_dead_letter::ReplayDeadLetter> for S {
    async fn handle(
        &self,
        _: pigeon_application::commands::replay_dead_letter::ReplayDeadLetter,
    ) -> Result<pigeon_domain::dead_letter::DeadLetter, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<pigeon_application::commands::retry_attempt::RetryAttempt> for S {
    async fn handle(
        &self,
        _: pigeon_application::commands::retry_attempt::RetryAttempt,
    ) -> Result<pigeon_domain::attempt::Attempt, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<pigeon_application::commands::send_test_event::SendTestEvent> for S {
    async fn handle(
        &self,
        _: pigeon_application::commands::send_test_event::SendTestEvent,
    ) -> Result<pigeon_application::commands::send_test_event::SendTestEventResult, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<GetApplicationById> for S {
    async fn handle(&self, _: GetApplicationById) -> Result<Option<Application>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<ListApplications> for S {
    async fn handle(
        &self,
        _: ListApplications,
    ) -> Result<PaginatedResult<Application>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<CreateEventType> for S {
    async fn handle(&self, _: CreateEventType) -> Result<EventType, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<UpdateEventType> for S {
    async fn handle(&self, _: UpdateEventType) -> Result<EventType, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<DeleteEventType> for S {
    async fn handle(&self, _: DeleteEventType) -> Result<(), ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<GetEventTypeById> for S {
    async fn handle(
        &self,
        _: GetEventTypeById,
    ) -> Result<Option<EventType>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<ListEventTypesByApp> for S {
    async fn handle(
        &self,
        _: ListEventTypesByApp,
    ) -> Result<PaginatedResult<EventType>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<CreateEndpoint> for S {
    async fn handle(&self, _: CreateEndpoint) -> Result<Endpoint, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<UpdateEndpoint> for S {
    async fn handle(&self, _: UpdateEndpoint) -> Result<Endpoint, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<DeleteEndpoint> for S {
    async fn handle(&self, _: DeleteEndpoint) -> Result<(), ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<GetEndpointById> for S {
    async fn handle(&self, _: GetEndpointById) -> Result<Option<Endpoint>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<ListEndpointsByApp> for S {
    async fn handle(
        &self,
        _: ListEndpointsByApp,
    ) -> Result<PaginatedResult<Endpoint>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl pigeon_application::ports::health::HealthChecker for S {
    async fn check(&self) -> bool {
        true
    }
}
#[async_trait]
impl CommandHandler<pigeon_application::commands::create_organization::CreateOrganization> for S {
    async fn handle(
        &self,
        _: pigeon_application::commands::create_organization::CreateOrganization,
    ) -> Result<Organization, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<UpdateOrganization> for S {
    async fn handle(&self, _: UpdateOrganization) -> Result<Organization, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<DeleteOrganization> for S {
    async fn handle(&self, _: DeleteOrganization) -> Result<(), ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<GetOrganizationById> for S {
    async fn handle(
        &self,
        _: GetOrganizationById,
    ) -> Result<Option<Organization>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<ListOrganizations> for S {
    async fn handle(
        &self,
        _: ListOrganizations,
    ) -> Result<PaginatedResult<Organization>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<CreateOidcConfig> for S {
    async fn handle(&self, _: CreateOidcConfig) -> Result<OidcConfig, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl CommandHandler<DeleteOidcConfig> for S {
    async fn handle(&self, _: DeleteOidcConfig) -> Result<(), ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<GetOidcConfigById> for S {
    async fn handle(&self, _: GetOidcConfigById) -> Result<Option<OidcConfig>, ApplicationError> {
        unimplemented!()
    }
}
#[async_trait]
impl QueryHandler<ListOidcConfigsByOrg> for S {
    async fn handle(
        &self,
        _: ListOidcConfigsByOrg,
    ) -> Result<PaginatedResult<OidcConfig>, ApplicationError> {
        unimplemented!()
    }
}

fn test_state() -> AppState {
    AppState {
        create_application: Arc::new(S),
        update_application: Arc::new(S),
        delete_application: Arc::new(S),
        send_message: Arc::new(S),
        get_application: Arc::new(S),
        list_applications: Arc::new(S),
        create_event_type: Arc::new(S),
        update_event_type: Arc::new(S),
        delete_event_type: Arc::new(S),
        get_event_type: Arc::new(S),
        list_event_types: Arc::new(S),
        create_endpoint: Arc::new(S),
        update_endpoint: Arc::new(S),
        delete_endpoint: Arc::new(S),
        get_endpoint: Arc::new(S),
        list_endpoints: Arc::new(S),
        health_checker: Arc::new(S),
        create_organization: Arc::new(S),
        update_organization: Arc::new(S),
        delete_organization: Arc::new(S),
        get_organization: Arc::new(S),
        list_organizations: Arc::new(S),
        create_oidc_config: Arc::new(S),
        delete_oidc_config: Arc::new(S),
        get_oidc_config: Arc::new(S),
        list_oidc_configs: Arc::new(S),
        oidc_config_read_store: Arc::new(StubOidcConfigReadStore),
        org_read_store: Arc::new(StubOrganizationReadStore),
        app_read_store: Arc::new(StubApplicationReadStore),
        jwks_provider: Arc::new(StubJwksProvider),
        replay_dead_letter: Arc::new(S),
        retry_attempt: Arc::new(S),
        send_test_event: Arc::new(S),
        metrics_render: Arc::new(|| String::new()),
        admin_org_id: None,
    }
}

#[tokio::test]
async fn api_route_without_auth_returns_401() {
    let app = crate::router(test_state());
    let req = Request::get("/api/v1/applications")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_route_skips_auth() {
    let app = crate::router(test_state());
    let req = Request::get("/health").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_ready_route_skips_auth() {
    let app = crate::router(test_state());
    let req = Request::get("/health/ready").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn openapi_route_skips_auth() {
    let app = crate::router(test_state());
    let req = Request::get("/api/openapi.json")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_bearer_token_returns_401() {
    let app = crate::router(test_state());
    let req = Request::get("/api/v1/applications")
        .header("authorization", "Bearer not-a-valid-jwt")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn malformed_authorization_header_returns_401() {
    let app = crate::router(test_state());
    let req = Request::get("/api/v1/applications")
        .header("authorization", "Basic dXNlcjpwYXNz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
