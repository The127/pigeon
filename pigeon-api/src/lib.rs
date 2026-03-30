pub mod auth;
pub mod dto;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod state;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod auth_tests;

use axum::routing::{delete, get, post};
use axum::Router;
use utoipa::OpenApi;

use crate::dto::application::{
    ApplicationResponse, CreateApplicationRequest, UpdateApplicationRequest,
};
use crate::dto::endpoint::{
    CreateEndpointRequest, EndpointResponse, UpdateEndpointRequest,
};
use crate::dto::event_type::{
    CreateEventTypeRequest, EventTypeResponse, UpdateEventTypeRequest,
};
use crate::handlers::audit;
use crate::handlers::dead_letters;
use crate::dto::attempt::AttemptResponse;
use crate::dto::dead_letter::DeadLetterResponse;
use crate::dto::message::{MessageResponse, SendMessageRequest, SendMessageResponse};
use crate::dto::oidc_config::{CreateOidcConfigRequest, OidcConfigResponse};
use crate::dto::organization::{
    CreateOrganizationRequest, OrganizationResponse, UpdateOrganizationRequest,
};
use crate::dto::pagination::PaginatedResponse;
use crate::error::ErrorBody;
use crate::handlers::{applications, endpoint_stats, endpoints, event_type_stats, event_types, health, messages, oidc_configs, organizations, stats};
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        applications::create_application,
        applications::get_application,
        applications::list_applications,
        applications::update_application,
        applications::delete_application,
        event_types::create_event_type,
        event_types::get_event_type,
        event_types::list_event_types,
        event_types::update_event_type,
        event_types::delete_event_type,
        endpoints::create_endpoint,
        endpoints::get_endpoint,
        endpoints::list_endpoints,
        endpoints::update_endpoint,
        endpoints::delete_endpoint,
        endpoints::rotate_signing_secret,
        endpoints::revoke_signing_secret,
        messages::send_message,
        messages::list_messages,
        messages::get_message,
        messages::list_attempts,
        messages::retrigger_message,
        dead_letters::list_dead_letters,
        dead_letters::get_dead_letter,
        dead_letters::replay,
        audit::list_audit_log,
        stats::get_stats,
        event_type_stats::get_event_type_stats,
        endpoint_stats::get_endpoint_stats,
        organizations::create_organization,
        organizations::get_organization,
        organizations::list_organizations,
        organizations::update_organization,
        organizations::delete_organization,
        oidc_configs::create_oidc_config,
        oidc_configs::get_oidc_config,
        oidc_configs::list_oidc_configs,
        oidc_configs::delete_oidc_config,
        oidc_configs::list_my_oidc_configs,
        oidc_configs::create_my_oidc_config,
        oidc_configs::delete_my_oidc_config,
        health::liveness,
        health::readiness,
    ),
    components(schemas(
        CreateApplicationRequest,
        UpdateApplicationRequest,
        ApplicationResponse,
        CreateEventTypeRequest,
        UpdateEventTypeRequest,
        EventTypeResponse,
        CreateEndpointRequest,
        UpdateEndpointRequest,
        EndpointResponse,
        SendMessageRequest,
        SendMessageResponse,
        MessageResponse,
        AttemptResponse,
        DeadLetterResponse,
        dead_letters::ReplayDeadLetterResponse,
        dto::stats::AppStatsResponse,
        dto::stats::TimeBucketResponse,
        dto::event_type_stats::EventTypeStatsResponse,
        dto::event_type_stats::EventTypeTimeBucketResponse,
        dto::event_type_stats::RecentMessageResponse,
        dto::endpoint_stats::EndpointStatsResponse,
        dto::endpoint_stats::EndpointTimeBucketResponse,
        CreateOrganizationRequest,
        UpdateOrganizationRequest,
        OrganizationResponse,
        CreateOidcConfigRequest,
        OidcConfigResponse,
        dto::audit::AuditLogResponse,
        PaginatedResponse,
        ErrorBody,
        health::HealthResponse,
    ))
)]
struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let app_routes = Router::new()
        .route("/", post(applications::create_application).get(applications::list_applications))
        .route(
            "/{id}",
            get(applications::get_application)
                .put(applications::update_application)
                .delete(applications::delete_application),
        )
        .route(
            "/{app_id}/event-types",
            post(event_types::create_event_type).get(event_types::list_event_types),
        )
        .route(
            "/{app_id}/event-types/{id}",
            get(event_types::get_event_type)
                .put(event_types::update_event_type)
                .delete(event_types::delete_event_type),
        )
        .route(
            "/{app_id}/event-types/{id}/stats",
            get(event_type_stats::get_event_type_stats),
        )
        .route(
            "/{app_id}/endpoints",
            post(endpoints::create_endpoint).get(endpoints::list_endpoints),
        )
        .route(
            "/{app_id}/endpoints/{id}",
            get(endpoints::get_endpoint)
                .put(endpoints::update_endpoint)
                .delete(endpoints::delete_endpoint),
        )
        .route(
            "/{app_id}/endpoints/{id}/stats",
            get(endpoint_stats::get_endpoint_stats),
        )
        .route("/{app_id}/stats", get(handlers::stats::get_stats))
        .route(
            "/{app_id}/messages",
            post(messages::send_message).get(messages::list_messages),
        )
        .route(
            "/{app_id}/messages/{id}",
            get(messages::get_message),
        )
        .route(
            "/{app_id}/messages/{msg_id}/attempts",
            get(messages::list_attempts),
        )
        .route(
            "/{app_id}/messages/{id}/retrigger",
            post(messages::retrigger_message),
        )
        .route(
            "/{app_id}/dead-letters",
            get(handlers::dead_letters::list_dead_letters),
        )
        .route(
            "/{app_id}/dead-letters/{id}",
            get(handlers::dead_letters::get_dead_letter),
        )
        .route(
            "/{app_id}/dead-letters/{id}/replay",
            post(handlers::dead_letters::replay),
        )
        .route(
            "/{app_id}/attempts/{id}/retry",
            post(handlers::attempts::retry),
        )
        .route(
            "/{app_id}/endpoints/{id}/test",
            post(handlers::test_event::send_test_event),
        )
        .route(
            "/{app_id}/endpoints/{id}/rotate",
            post(endpoints::rotate_signing_secret),
        )
        .route(
            "/{app_id}/endpoints/{id}/secrets/{index}",
            delete(endpoints::revoke_signing_secret),
        );

    // API routes protected by JWT auth middleware
    let api_routes = Router::new()
        .nest("/api/v1/applications", app_routes)
        .route("/api/v1/audit-log", get(audit::list_audit_log))
        .route(
            "/api/v1/oidc-configs",
            get(oidc_configs::list_my_oidc_configs).post(oidc_configs::create_my_oidc_config),
        )
        .route(
            "/api/v1/oidc-configs/{id}",
            delete(oidc_configs::delete_my_oidc_config),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    let org_routes = Router::new()
        .route(
            "/",
            post(organizations::create_organization).get(organizations::list_organizations),
        )
        .route(
            "/{id}",
            get(organizations::get_organization)
                .put(organizations::update_organization)
                .delete(organizations::delete_organization),
        );

    let oidc_config_routes = Router::new()
        .route(
            "/",
            post(oidc_configs::create_oidc_config).get(oidc_configs::list_oidc_configs),
        )
        .route(
            "/{id}",
            get(oidc_configs::get_oidc_config).delete(oidc_configs::delete_oidc_config),
        );

    // Admin routes: JWT auth + bootstrap org restriction.
    // Layers execute bottom-to-top: JWT auth runs first, then org check.
    let admin_routes = Router::new()
        .nest("/admin/v1/organizations", org_routes)
        .nest(
            "/admin/v1/organizations/{org_id}/oidc-configs",
            oidc_config_routes,
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::admin_org_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    Router::new()
        .merge(api_routes)
        .merge(admin_routes)
        .route("/api/openapi.json", get(|| async { axum::Json(ApiDoc::openapi()) }))
        .route("/api/v1/auth/config", get(handlers::auth_config::auth_config))
        .route("/health", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .route("/metrics", get(handlers::metrics::render))
        .layer(axum::middleware::from_fn(middleware::request_id))
        .with_state(state)
}

/// Test-only router that uses a header-based test auth middleware instead of JWT validation.
/// Test-only header; production uses JWT auth.
/// Used by handler unit tests that test handler logic in isolation.
#[cfg(test)]
pub(crate) fn router_without_auth(state: AppState) -> Router {
    let app_routes = Router::new()
        .route("/", post(applications::create_application).get(applications::list_applications))
        .route(
            "/{id}",
            get(applications::get_application)
                .put(applications::update_application)
                .delete(applications::delete_application),
        )
        .route(
            "/{app_id}/event-types",
            post(event_types::create_event_type).get(event_types::list_event_types),
        )
        .route(
            "/{app_id}/event-types/{id}",
            get(event_types::get_event_type)
                .put(event_types::update_event_type)
                .delete(event_types::delete_event_type),
        )
        .route(
            "/{app_id}/event-types/{id}/stats",
            get(event_type_stats::get_event_type_stats),
        )
        .route(
            "/{app_id}/endpoints",
            post(endpoints::create_endpoint).get(endpoints::list_endpoints),
        )
        .route(
            "/{app_id}/endpoints/{id}",
            get(endpoints::get_endpoint)
                .put(endpoints::update_endpoint)
                .delete(endpoints::delete_endpoint),
        )
        .route(
            "/{app_id}/endpoints/{id}/stats",
            get(endpoint_stats::get_endpoint_stats),
        )
        .route("/{app_id}/stats", get(handlers::stats::get_stats))
        .route(
            "/{app_id}/messages",
            post(messages::send_message).get(messages::list_messages),
        )
        .route(
            "/{app_id}/messages/{id}",
            get(messages::get_message),
        )
        .route(
            "/{app_id}/messages/{msg_id}/attempts",
            get(messages::list_attempts),
        )
        .route(
            "/{app_id}/messages/{id}/retrigger",
            post(messages::retrigger_message),
        )
        .route(
            "/{app_id}/dead-letters",
            get(handlers::dead_letters::list_dead_letters),
        )
        .route(
            "/{app_id}/dead-letters/{id}",
            get(handlers::dead_letters::get_dead_letter),
        )
        .route(
            "/{app_id}/dead-letters/{id}/replay",
            post(handlers::dead_letters::replay),
        )
        .route(
            "/{app_id}/attempts/{id}/retry",
            post(handlers::attempts::retry),
        )
        .route(
            "/{app_id}/endpoints/{id}/test",
            post(handlers::test_event::send_test_event),
        )
        .route(
            "/{app_id}/endpoints/{id}/rotate",
            post(endpoints::rotate_signing_secret),
        )
        .route(
            "/{app_id}/endpoints/{id}/secrets/{index}",
            delete(endpoints::revoke_signing_secret),
        );

    let api_routes = Router::new()
        .nest("/api/v1/applications", app_routes)
        .route("/api/v1/audit-log", get(audit::list_audit_log))
        .route(
            "/api/v1/oidc-configs",
            get(oidc_configs::list_my_oidc_configs).post(oidc_configs::create_my_oidc_config),
        )
        .route(
            "/api/v1/oidc-configs/{id}",
            delete(oidc_configs::delete_my_oidc_config),
        )
        .layer(axum::middleware::from_fn(
            test_support::test_auth_middleware,
        ));

    let org_routes = Router::new()
        .route(
            "/",
            post(organizations::create_organization).get(organizations::list_organizations),
        )
        .route(
            "/{id}",
            get(organizations::get_organization)
                .put(organizations::update_organization)
                .delete(organizations::delete_organization),
        );

    let oidc_config_routes = Router::new()
        .route(
            "/",
            post(oidc_configs::create_oidc_config).get(oidc_configs::list_oidc_configs),
        )
        .route(
            "/{id}",
            get(oidc_configs::get_oidc_config).delete(oidc_configs::delete_oidc_config),
        );

    let admin_routes = Router::new()
        .nest("/admin/v1/organizations", org_routes)
        .nest("/admin/v1/organizations/{org_id}/oidc-configs", oidc_config_routes)
        .layer(axum::middleware::from_fn(
            test_support::test_auth_middleware,
        ));

    Router::new()
        .merge(api_routes)
        .merge(admin_routes)
        .route("/api/v1/auth/config", get(handlers::auth_config::auth_config))
        .route("/api/openapi.json", get(|| async { axum::Json(ApiDoc::openapi()) }))
        .route("/health", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .route("/metrics", get(handlers::metrics::render))
        .layer(axum::middleware::from_fn(middleware::request_id))
        .with_state(state)
}
