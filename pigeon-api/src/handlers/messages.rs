use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use pigeon_application::commands::retrigger_message::RetriggerMessage;
use pigeon_application::commands::send_message::SendMessage;
use pigeon_application::queries::get_message_by_id::GetMessageById;
use pigeon_application::queries::list_attempts_by_message::ListAttemptsByMessage;
use pigeon_application::queries::list_messages_by_app::ListMessagesByApp;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::message::MessageId;

use crate::dto::attempt::AttemptResponse;
use crate::dto::message::{MessageResponse, SendMessageRequest, SendMessageResponse};
use crate::dto::pagination::MessageListQuery;
use crate::error::{ApiError, ErrorBody};
use crate::extractors::{AuthInfo, OrgId};
use crate::state::AppState;
use pigeon_application::mediator::dispatcher::dispatch;

use super::verify_app_ownership;

/// Send a message to an application's endpoints
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/messages",
    params(("app_id" = Uuid, Path, description = "Application ID")),
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Message sent", body = SendMessageResponse),
        (status = 200, description = "Duplicate message (idempotent)", body = SendMessageResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
    tag = "messages"
)]
pub async fn send_message(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path(app_id): Path<Uuid>,
    Json(body): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &auth.org_id).await?;

    let command = SendMessage {
        org_id: auth.org_id.clone(),
        app_id,
        event_type_id: EventTypeId::from_uuid(body.event_type_id),
        payload: body.payload,
        idempotency_key: body.idempotency_key,
    };

    let result = dispatch(&*state.send_message, command, &auth.user_id, &auth.org_id, &*state.audit_store).await.map_err(ApiError)?;
    let was_duplicate = result.was_duplicate;

    if was_duplicate {
        metrics::counter!("pigeon_messages_total", "status" => "duplicate").increment(1);
    } else {
        metrics::counter!("pigeon_messages_total", "status" => "new").increment(1);
    }

    let response = SendMessageResponse::from(result);

    let status = if was_duplicate {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };

    Ok((status, Json(response)))
}

/// List messages for an application
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/messages",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        MessageListQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of messages"),
    ),
    tag = "messages"
)]
pub async fn list_messages(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path(app_id): Path<Uuid>,
    Query(query): Query<MessageListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let result = state
        .list_messages
        .handle(ListMessagesByApp {
            app_id,
            org_id,
            event_type_id: query.event_type_id.map(EventTypeId::from_uuid),
            offset: query.offset.unwrap_or(0),
            limit: query.limit.unwrap_or(20),
        })
        .await
        .map_err(ApiError)?;

    let response = serde_json::json!({
        "items": result.items.into_iter().map(MessageResponse::from).collect::<Vec<_>>(),
        "total": result.total,
        "offset": result.offset,
        "limit": result.limit,
    });

    Ok(Json(response))
}

/// Get a message by ID
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/messages/{id}",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Message ID"),
    ),
    responses(
        (status = 200, description = "Message found", body = MessageResponse),
        (status = 404, description = "Message not found", body = ErrorBody),
    ),
    tag = "messages"
)]
pub async fn get_message(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let msg = state
        .get_message
        .handle(GetMessageById {
            id: MessageId::from_uuid(id),
            org_id,
        })
        .await
        .map_err(ApiError)?
        .ok_or(ApiError(pigeon_application::error::ApplicationError::NotFound))?;

    Ok(Json(MessageResponse::from(msg)))
}

/// List delivery attempts for a message
#[utoipa::path(
    get,
    path = "/api/v1/applications/{app_id}/messages/{msg_id}/attempts",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("msg_id" = Uuid, Path, description = "Message ID"),
    ),
    responses(
        (status = 200, description = "List of attempts", body = Vec<AttemptResponse>),
    ),
    tag = "messages"
)]
pub async fn list_attempts(
    State(state): State<AppState>,
    OrgId(org_id): OrgId,
    Path((app_id, msg_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &org_id).await?;

    let attempts = state
        .list_attempts
        .handle(ListAttemptsByMessage {
            message_id: MessageId::from_uuid(msg_id),
            org_id,
        })
        .await
        .map_err(ApiError)?;

    let response: Vec<AttemptResponse> = attempts.into_iter().map(AttemptResponse::from).collect();
    Ok(Json(response))
}

/// Retrigger a message, creating new delivery attempts for currently matching endpoints
#[utoipa::path(
    post,
    path = "/api/v1/applications/{app_id}/messages/{id}/retrigger",
    params(
        ("app_id" = Uuid, Path, description = "Application ID"),
        ("id" = Uuid, Path, description = "Message ID"),
    ),
    responses(
        (status = 200, description = "Message retriggered", body = SendMessageResponse),
        (status = 404, description = "Message not found", body = ErrorBody),
        (status = 400, description = "No matching endpoints", body = ErrorBody),
    ),
    tag = "messages"
)]
pub async fn retrigger_message(
    State(state): State<AppState>,
    auth: AuthInfo,
    Path((app_id, id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let app_id = ApplicationId::from_uuid(app_id);
    verify_app_ownership(&*state.app_read_store, &app_id, &auth.org_id).await?;

    let result = dispatch(
        &*state.retrigger_message,
        RetriggerMessage {
            message_id: MessageId::from_uuid(id),
            org_id: auth.org_id.clone(),
        },
        &auth.user_id,
        &auth.org_id,
        &*state.audit_store,
    )
    .await
    .map_err(ApiError)?;

    let response = serde_json::json!({
        "message_id": result.message.id().as_uuid(),
        "attempts_created": result.attempts_created,
    });

    Ok(Json(response))
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
    use pigeon_application::commands::create_organization::CreateOrganization;
    use pigeon_application::commands::delete_organization::DeleteOrganization;
    use pigeon_application::commands::update_organization::UpdateOrganization;
    use pigeon_application::queries::get_organization_by_id::GetOrganizationById;
    use pigeon_application::queries::list_organizations::ListOrganizations;
    use pigeon_domain::message::Message;
    use pigeon_domain::organization::Organization;
    use std::sync::Arc;
    use tower::ServiceExt;

    // --- Fakes for existing handlers (minimal stubs) ---

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
        async fn handle(
            &self,
            _q: GetApplicationById,
        ) -> Result<Option<Application>, ApplicationError> {
            Ok(None)
        }
    }

    struct StubListHandler;
    #[async_trait]
    impl QueryHandler<ListApplications> for StubListHandler {
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

    // --- Send message fakes ---

    struct FakeSendMessageHandler {
        result: Result<SendMessageResult, ApplicationError>,
    }

    #[async_trait]
    impl CommandHandler<SendMessage> for FakeSendMessageHandler {
        async fn handle(&self, command: SendMessage) -> Result<SendMessageResult, ApplicationError> {
            match &self.result {
                Ok(r) => {
                    // Build a real message from the command to return realistic data
                    let msg = Message::new(
                        command.app_id,
                        command.event_type_id,
                        command.payload,
                        command.idempotency_key,
                        chrono::Duration::hours(24),
                    )
                    .map_err(|e| ApplicationError::Validation(e.to_string()))?;

                    Ok(SendMessageResult {
                        message: msg,
                        attempts_created: r.attempts_created,
                        was_duplicate: r.was_duplicate,
                    })
                }
                Err(e) => Err(match e {
                    ApplicationError::Validation(s) => ApplicationError::Validation(s.clone()),
                    _ => ApplicationError::Internal("error".into()),
                }),
            }
        }
    }

    fn build_state(send: impl CommandHandler<SendMessage> + 'static, app_read_store: Arc<dyn pigeon_application::ports::stores::ApplicationReadStore>) -> AppState {
        use crate::test_support::*;
        AppState {
            create_application: Arc::new(StubCreateHandler),
            update_application: Arc::new(StubUpdateHandler),
            delete_application: Arc::new(StubDeleteHandler),
            get_application: Arc::new(StubGetHandler),
            list_applications: Arc::new(StubListHandler),
            send_message: Arc::new(send),
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
            app_read_store,
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

    fn test_router(state: AppState) -> Router {
        crate::router_without_auth(state)
    }

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    use pigeon_domain::application::ApplicationState as DomainAppState;
    use pigeon_domain::organization::OrganizationId;

    const TEST_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

    fn test_org_id() -> OrganizationId {
        OrganizationId::from_uuid(
            Uuid::parse_str(TEST_ORG_ID).unwrap(),
        )
    }

    fn fake_app_for(app_id: &pigeon_domain::application::ApplicationId) -> Application {
        Application::reconstitute(DomainAppState {
            id: app_id.clone(),
            org_id: test_org_id(),
            name: "test-app".to_string(),
            uid: format!("app_{}", Uuid::new_v4()),
            created_at: chrono::Utc::now(),
            version: pigeon_domain::version::Version::new(0),
        })
    }

    fn dummy_send_result(attempts: usize, duplicate: bool) -> SendMessageResult {
        let msg = pigeon_domain::test_support::any_message();
        SendMessageResult {
            message: msg,
            attempts_created: attempts,
            was_duplicate: duplicate,
        }
    }

    // --- Tests ---

    #[tokio::test]
    async fn send_new_message_returns_201() {
        let app_id = pigeon_domain::application::ApplicationId::new();
        let state = build_state(
            FakeSendMessageHandler {
                result: Ok(dummy_send_result(2, false)),
            },
            Arc::new(crate::test_support::FakeApplicationReadStore {
                app: Some(fake_app_for(&app_id)),
            }),
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/applications/{}/messages", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "event_type_id": Uuid::new_v4(),
                            "payload": {"user": "u1"},
                            "idempotency_key": "key-1"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let json = body_json(response.into_body()).await;
        assert!(json["id"].is_string());
        assert_eq!(json["was_duplicate"], false);
    }

    #[tokio::test]
    async fn send_duplicate_message_returns_200() {
        let app_id = pigeon_domain::application::ApplicationId::new();
        let state = build_state(
            FakeSendMessageHandler {
                result: Ok(dummy_send_result(0, true)),
            },
            Arc::new(crate::test_support::FakeApplicationReadStore {
                app: Some(fake_app_for(&app_id)),
            }),
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/applications/{}/messages", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "event_type_id": Uuid::new_v4(),
                            "payload": {"user": "u1"},
                            "idempotency_key": "dup-key"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response.into_body()).await;
        assert_eq!(json["was_duplicate"], true);
        assert_eq!(json["attempts_created"], 0);
    }

    #[tokio::test]
    async fn send_invalid_payload_returns_400() {
        let app_id = pigeon_domain::application::ApplicationId::new();
        let state = build_state(
            FakeSendMessageHandler {
                result: Ok(dummy_send_result(0, false)),
            },
            Arc::new(crate::test_support::FakeApplicationReadStore {
                app: Some(fake_app_for(&app_id)),
            }),
        );
        let router = test_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/applications/{}/messages", app_id.as_uuid()))
                    .header("content-type", "application/json")
                    .header("x-org-id", TEST_ORG_ID)
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "event_type_id": Uuid::new_v4(),
                            "payload": "not an object"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
