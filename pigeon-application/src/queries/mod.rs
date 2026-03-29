pub mod get_app_stats;
pub mod get_application_by_id;
pub mod get_dead_letter_by_id;
pub mod get_endpoint_by_id;
pub mod get_event_type_by_id;
pub mod get_message_by_id;
pub mod get_oidc_config_by_id;
pub mod get_organization_by_id;
pub mod list_applications;
pub mod list_attempts_by_message;
pub mod list_dead_letters_by_app;
pub mod list_endpoints_by_app;
pub mod list_event_types_by_app;
pub mod list_messages_by_app;
pub mod list_oidc_configs_by_org;
pub mod list_organizations;

#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}
