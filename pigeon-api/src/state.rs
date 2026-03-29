use std::sync::Arc;

use pigeon_application::commands::create_application::CreateApplication;
use pigeon_application::commands::replay_dead_letter::ReplayDeadLetter;
use pigeon_application::commands::retry_attempt::RetryAttempt;
use pigeon_application::commands::send_test_event::SendTestEvent;
use pigeon_domain::organization::OrganizationId;
use pigeon_application::commands::create_endpoint::CreateEndpoint;
use pigeon_application::commands::create_event_type::CreateEventType;
use pigeon_application::commands::create_oidc_config::CreateOidcConfig;
use pigeon_application::commands::create_organization::CreateOrganization;
use pigeon_application::commands::delete_application::DeleteApplication;
use pigeon_application::commands::delete_endpoint::DeleteEndpoint;
use pigeon_application::commands::delete_event_type::DeleteEventType;
use pigeon_application::commands::delete_oidc_config::DeleteOidcConfig;
use pigeon_application::commands::delete_organization::DeleteOrganization;
use pigeon_application::commands::send_message::SendMessage;
use pigeon_application::commands::update_application::UpdateApplication;
use pigeon_application::commands::update_endpoint::UpdateEndpoint;
use pigeon_application::commands::update_event_type::UpdateEventType;
use pigeon_application::commands::update_organization::UpdateOrganization;
use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
use pigeon_application::ports::health::HealthChecker;
use pigeon_application::ports::stores::{ApplicationReadStore, OidcConfigReadStore, OrganizationReadStore};
use pigeon_application::queries::get_application_by_id::GetApplicationById;
use pigeon_application::queries::get_endpoint_by_id::GetEndpointById;
use pigeon_application::queries::get_event_type_by_id::GetEventTypeById;
use pigeon_application::queries::get_oidc_config_by_id::GetOidcConfigById;
use pigeon_application::queries::get_organization_by_id::GetOrganizationById;
use pigeon_application::queries::list_applications::ListApplications;
use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByApp;
use pigeon_application::queries::list_event_types_by_app::ListEventTypesByApp;
use pigeon_application::queries::list_oidc_configs_by_org::ListOidcConfigsByOrg;
use pigeon_application::queries::get_dead_letter_by_id::GetDeadLetterById;
use pigeon_application::queries::get_message_by_id::GetMessageById;
use pigeon_application::queries::list_attempts_by_message::ListAttemptsByMessage;
use pigeon_application::queries::list_dead_letters_by_app::ListDeadLettersByApp;
use pigeon_application::queries::list_messages_by_app::ListMessagesByApp;
use pigeon_application::queries::list_organizations::ListOrganizations;

use crate::auth::JwksProvider;

#[derive(Clone)]
pub struct AppState {
    pub create_application: Arc<dyn CommandHandler<CreateApplication>>,
    pub update_application: Arc<dyn CommandHandler<UpdateApplication>>,
    pub delete_application: Arc<dyn CommandHandler<DeleteApplication>>,
    pub send_message: Arc<dyn CommandHandler<SendMessage>>,
    pub get_application: Arc<dyn QueryHandler<GetApplicationById>>,
    pub list_applications: Arc<dyn QueryHandler<ListApplications>>,
    pub create_event_type: Arc<dyn CommandHandler<CreateEventType>>,
    pub update_event_type: Arc<dyn CommandHandler<UpdateEventType>>,
    pub delete_event_type: Arc<dyn CommandHandler<DeleteEventType>>,
    pub get_event_type: Arc<dyn QueryHandler<GetEventTypeById>>,
    pub list_event_types: Arc<dyn QueryHandler<ListEventTypesByApp>>,
    pub create_endpoint: Arc<dyn CommandHandler<CreateEndpoint>>,
    pub update_endpoint: Arc<dyn CommandHandler<UpdateEndpoint>>,
    pub delete_endpoint: Arc<dyn CommandHandler<DeleteEndpoint>>,
    pub get_endpoint: Arc<dyn QueryHandler<GetEndpointById>>,
    pub list_endpoints: Arc<dyn QueryHandler<ListEndpointsByApp>>,
    pub get_message: Arc<dyn QueryHandler<GetMessageById>>,
    pub list_messages: Arc<dyn QueryHandler<ListMessagesByApp>>,
    pub list_attempts: Arc<dyn QueryHandler<ListAttemptsByMessage>>,
    pub get_dead_letter: Arc<dyn QueryHandler<GetDeadLetterById>>,
    pub list_dead_letters: Arc<dyn QueryHandler<ListDeadLettersByApp>>,
    pub replay_dead_letter: Arc<dyn CommandHandler<ReplayDeadLetter>>,
    pub retry_attempt: Arc<dyn CommandHandler<RetryAttempt>>,
    pub send_test_event: Arc<dyn CommandHandler<SendTestEvent>>,
    pub health_checker: Arc<dyn HealthChecker>,
    pub create_organization: Arc<dyn CommandHandler<CreateOrganization>>,
    pub update_organization: Arc<dyn CommandHandler<UpdateOrganization>>,
    pub delete_organization: Arc<dyn CommandHandler<DeleteOrganization>>,
    pub get_organization: Arc<dyn QueryHandler<GetOrganizationById>>,
    pub list_organizations: Arc<dyn QueryHandler<ListOrganizations>>,
    pub create_oidc_config: Arc<dyn CommandHandler<CreateOidcConfig>>,
    pub delete_oidc_config: Arc<dyn CommandHandler<DeleteOidcConfig>>,
    pub get_oidc_config: Arc<dyn QueryHandler<GetOidcConfigById>>,
    pub list_oidc_configs: Arc<dyn QueryHandler<ListOidcConfigsByOrg>>,
    pub oidc_config_read_store: Arc<dyn OidcConfigReadStore>,
    pub org_read_store: Arc<dyn OrganizationReadStore>,
    pub app_read_store: Arc<dyn ApplicationReadStore>,
    pub jwks_provider: Arc<dyn JwksProvider>,
    pub metrics_render: Arc<dyn Fn() -> String + Send + Sync>,
    pub admin_org_id: Option<OrganizationId>,
}
