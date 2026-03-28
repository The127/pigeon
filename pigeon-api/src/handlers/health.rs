use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, body = HealthResponse),
    )
)]
pub async fn liveness() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, body = HealthResponse),
        (status = 503, body = HealthResponse),
    )
)]
pub async fn readiness(State(state): State<AppState>) -> impl IntoResponse {
    if state.health_checker.check().await {
        (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ready".to_string(),
            }),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "not ready".to_string(),
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use pigeon_application::error::ApplicationError;
    use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
    use pigeon_application::ports::health::HealthChecker;
    use pigeon_application::queries::PaginatedResult;
    use pigeon_domain::application::Application;
    use pigeon_domain::endpoint::Endpoint;
    use pigeon_domain::event_type::EventType;
    use pigeon_domain::organization::Organization;
    use tower::ServiceExt;

    use crate::state::AppState;

    struct AlwaysHealthy;
    #[async_trait]
    impl HealthChecker for AlwaysHealthy {
        async fn check(&self) -> bool {
            true
        }
    }

    struct AlwaysUnhealthy;
    #[async_trait]
    impl HealthChecker for AlwaysUnhealthy {
        async fn check(&self) -> bool {
            false
        }
    }

    // Minimal fakes to satisfy AppState — health tests don't call these
    struct StubCreate;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::create_application::CreateApplication>
        for StubCreate
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::create_application::CreateApplication,
        ) -> Result<Application, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubUpdate;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::update_application::UpdateApplication>
        for StubUpdate
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::update_application::UpdateApplication,
        ) -> Result<Application, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubDelete;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::delete_application::DeleteApplication>
        for StubDelete
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::delete_application::DeleteApplication,
        ) -> Result<(), ApplicationError> {
            unimplemented!()
        }
    }

    struct StubGet;
    #[async_trait]
    impl QueryHandler<pigeon_application::queries::get_application_by_id::GetApplicationById>
        for StubGet
    {
        async fn handle(
            &self,
            _: pigeon_application::queries::get_application_by_id::GetApplicationById,
        ) -> Result<Option<Application>, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubList;
    #[async_trait]
    impl QueryHandler<pigeon_application::queries::list_applications::ListApplications>
        for StubList
    {
        async fn handle(
            &self,
            _: pigeon_application::queries::list_applications::ListApplications,
        ) -> Result<PaginatedResult<Application>, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubSendMessage;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::send_message::SendMessage>
        for StubSendMessage
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::send_message::SendMessage,
        ) -> Result<
            pigeon_application::commands::send_message::SendMessageResult,
            ApplicationError,
        > {
            unimplemented!()
        }
    }

    struct StubCreateEt;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::create_event_type::CreateEventType>
        for StubCreateEt
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::create_event_type::CreateEventType,
        ) -> Result<EventType, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubUpdateEt;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::update_event_type::UpdateEventType>
        for StubUpdateEt
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::update_event_type::UpdateEventType,
        ) -> Result<EventType, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubDeleteEt;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::delete_event_type::DeleteEventType>
        for StubDeleteEt
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::delete_event_type::DeleteEventType,
        ) -> Result<(), ApplicationError> {
            unimplemented!()
        }
    }

    struct StubGetEt;
    #[async_trait]
    impl QueryHandler<pigeon_application::queries::get_event_type_by_id::GetEventTypeById>
        for StubGetEt
    {
        async fn handle(
            &self,
            _: pigeon_application::queries::get_event_type_by_id::GetEventTypeById,
        ) -> Result<Option<EventType>, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubListEt;
    #[async_trait]
    impl QueryHandler<pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp>
        for StubListEt
    {
        async fn handle(
            &self,
            _: pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp,
        ) -> Result<PaginatedResult<EventType>, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubCreateEp;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::create_endpoint::CreateEndpoint>
        for StubCreateEp
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::create_endpoint::CreateEndpoint,
        ) -> Result<Endpoint, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubUpdateEp;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::update_endpoint::UpdateEndpoint>
        for StubUpdateEp
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::update_endpoint::UpdateEndpoint,
        ) -> Result<Endpoint, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubDeleteEp;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::delete_endpoint::DeleteEndpoint>
        for StubDeleteEp
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::delete_endpoint::DeleteEndpoint,
        ) -> Result<(), ApplicationError> {
            unimplemented!()
        }
    }

    struct StubGetEp;
    #[async_trait]
    impl QueryHandler<pigeon_application::queries::get_endpoint_by_id::GetEndpointById>
        for StubGetEp
    {
        async fn handle(
            &self,
            _: pigeon_application::queries::get_endpoint_by_id::GetEndpointById,
        ) -> Result<Option<Endpoint>, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubListEp;
    #[async_trait]
    impl QueryHandler<pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp>
        for StubListEp
    {
        async fn handle(
            &self,
            _: pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp,
        ) -> Result<PaginatedResult<Endpoint>, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubCreateOrg;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::create_organization::CreateOrganization>
        for StubCreateOrg
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::create_organization::CreateOrganization,
        ) -> Result<Organization, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubUpdateOrg;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::update_organization::UpdateOrganization>
        for StubUpdateOrg
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::update_organization::UpdateOrganization,
        ) -> Result<Organization, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubDeleteOrg;
    #[async_trait]
    impl CommandHandler<pigeon_application::commands::delete_organization::DeleteOrganization>
        for StubDeleteOrg
    {
        async fn handle(
            &self,
            _: pigeon_application::commands::delete_organization::DeleteOrganization,
        ) -> Result<(), ApplicationError> {
            unimplemented!()
        }
    }

    struct StubGetOrg;
    #[async_trait]
    impl QueryHandler<pigeon_application::queries::get_organization_by_id::GetOrganizationById>
        for StubGetOrg
    {
        async fn handle(
            &self,
            _: pigeon_application::queries::get_organization_by_id::GetOrganizationById,
        ) -> Result<Option<Organization>, ApplicationError> {
            unimplemented!()
        }
    }

    struct StubListOrgs;
    #[async_trait]
    impl QueryHandler<pigeon_application::queries::list_organizations::ListOrganizations>
        for StubListOrgs
    {
        async fn handle(
            &self,
            _: pigeon_application::queries::list_organizations::ListOrganizations,
        ) -> Result<PaginatedResult<Organization>, ApplicationError> {
            unimplemented!()
        }
    }

    fn state_with_health(checker: Arc<dyn HealthChecker>) -> AppState {
        use crate::test_support::*;
        AppState {
            create_application: Arc::new(StubCreate),
            update_application: Arc::new(StubUpdate),
            delete_application: Arc::new(StubDelete),
            send_message: Arc::new(StubSendMessage),
            get_application: Arc::new(StubGet),
            list_applications: Arc::new(StubList),
            create_event_type: Arc::new(StubCreateEt),
            update_event_type: Arc::new(StubUpdateEt),
            delete_event_type: Arc::new(StubDeleteEt),
            get_event_type: Arc::new(StubGetEt),
            list_event_types: Arc::new(StubListEt),
            create_endpoint: Arc::new(StubCreateEp),
            update_endpoint: Arc::new(StubUpdateEp),
            delete_endpoint: Arc::new(StubDeleteEp),
            get_endpoint: Arc::new(StubGetEp),
            list_endpoints: Arc::new(StubListEp),
            health_checker: checker,
            create_organization: Arc::new(StubCreateOrg),
            update_organization: Arc::new(StubUpdateOrg),
            delete_organization: Arc::new(StubDeleteOrg),
            get_organization: Arc::new(StubGetOrg),
            list_organizations: Arc::new(StubListOrgs),
            create_oidc_config: Arc::new(StubCreateOidcConfigHandler),
            delete_oidc_config: Arc::new(StubDeleteOidcConfigHandler),
            get_oidc_config: Arc::new(StubGetOidcConfigHandler),
            list_oidc_configs: Arc::new(StubListOidcConfigsHandler),
            oidc_config_read_store: Arc::new(StubOidcConfigReadStore),
            app_read_store: Arc::new(StubApplicationReadStore),
            jwks_provider: Arc::new(StubJwksProvider),
            replay_dead_letter: Arc::new(StubReplayDeadLetterHandler),
            metrics_render: Arc::new(|| String::new()),
            admin_org_id: None,
        }
    }

    #[tokio::test]
    async fn liveness_returns_200() {
        let app = crate::router_without_auth(state_with_health(Arc::new(AlwaysHealthy)));
        let req = Request::get("/health").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn readiness_returns_200_when_healthy() {
        let app = crate::router_without_auth(state_with_health(Arc::new(AlwaysHealthy)));
        let req = Request::get("/health/ready").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn readiness_returns_503_when_unhealthy() {
        let app = crate::router_without_auth(state_with_health(Arc::new(AlwaysUnhealthy)));
        let req = Request::get("/health/ready").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
