use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use pigeon_application::commands::send_test_event::SendTestEvent;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::EndpointId;

use crate::error::{ApiError, ErrorBody};
use crate::extractors::AuthInfo;
use crate::state::AppState;
use pigeon_application::mediator::dispatcher::dispatch;

#[derive(Serialize, ToSchema)]
pub(crate) struct TestEventResponse {
    pub message_id: Uuid,
}

/// Send a test event to a specific endpoint
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/endpoints/{id}/test",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Endpoint ID"),
    ),
    responses(
        (status = 201, description = "Test event sent", body = TestEventResponse),
        (status = 404, description = "Application or endpoint not found", body = ErrorBody),
    ),
    tag = "endpoints"
)]
pub(crate) async fn send_test_event(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path((app_id, endpoint_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);

    let command = SendTestEvent {
        org_id: auth.org_id.clone(),
        app_id,
        endpoint_id: EndpointId::from_uuid(endpoint_id),
    };

    let result = dispatch(&*state.send_test_event, command, &auth.user_id, &auth.org_id, &*state.audit_store).await.map_err(ApiError)?;

    let response = TestEventResponse {
        message_id: *result.message.id().as_uuid(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use axum::Router;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    use pigeon_application::commands::create_application::CreateApplication;
    use pigeon_application::commands::create_endpoint::CreateEndpoint;
    use pigeon_application::commands::create_event_type::CreateEventType;
    use pigeon_application::commands::create_organization::CreateOrganization;
    use pigeon_application::commands::delete_application::DeleteApplication;
    use pigeon_application::commands::delete_endpoint::DeleteEndpoint;
    use pigeon_application::commands::delete_event_type::DeleteEventType;
    use pigeon_application::commands::delete_organization::DeleteOrganization;
    use pigeon_application::commands::send_message::{SendMessage, SendMessageResult};
    use pigeon_application::commands::send_test_event::{SendTestEvent, SendTestEventResult};
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
    use pigeon_domain::organization::Organization;

    use crate::state::AppState;

    // --- Stubs ---

    struct StubCreateHandler;
    #[async_trait]
    impl CommandHandler<CreateApplication> for StubCreateHandler {
        async fn handle(&self, _c: CreateApplication) -> Result<Application, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubUpdateHandler;
    #[async_trait]
    impl CommandHandler<UpdateApplication> for StubUpdateHandler {
        async fn handle(&self, _c: UpdateApplication) -> Result<Application, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
        }
    }

    struct StubDeleteHandler;
    #[async_trait]
    impl CommandHandler<DeleteApplication> for StubDeleteHandler {
        async fn handle(&self, _c: DeleteApplication) -> Result<(), ApplicationError> {
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

    struct StubSendMessageHandler;
    #[async_trait]
    impl CommandHandler<SendMessage> for StubSendMessageHandler {
        async fn handle(&self, _c: SendMessage) -> Result<SendMessageResult, ApplicationError> {
            Err(ApplicationError::Internal("stub".into()))
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

    // --- SendTestEvent fake ---

    struct FakeSendTestEventHandler;
    #[async_trait]
    impl CommandHandler<SendTestEvent> for FakeSendTestEventHandler {
        async fn handle(&self, _c: SendTestEvent) -> Result<SendTestEventResult, ApplicationError> {
            let message = pigeon_domain::test_support::any_message();
            Ok(SendTestEventResult { message })
        }
    }

    const TEST_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

    fn build_state(
        send_test_event: impl CommandHandler<SendTestEvent> + 'static,
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
            send_test_event: Arc::new(send_test_event),
            list_audit_log: Arc::new(StubListAuditLogHandler),
            audit_store: Arc::new(StubAuditStore),
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
    async fn send_test_event_returns_201() {
        let app_id = pigeon_domain::application::ApplicationId::new();
        let endpoint_id = pigeon_domain::endpoint::EndpointId::new();
        let state = build_state(
            FakeSendTestEventHandler,
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/v1/applications/{}/endpoints/{}/test",
                        app_id.as_uuid(),
                        endpoint_id.as_uuid()
                    ))
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::CREATED);
        let json = body_json(response.into_body()).await;
        assert!(json["message_id"].is_string());
    }

}
