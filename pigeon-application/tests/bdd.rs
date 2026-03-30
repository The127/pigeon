use std::sync::Arc;

use cucumber::{given, then, when, World};
use pigeon_application::commands::create_application::{
    CreateApplication, CreateApplicationHandler,
};
use pigeon_application::commands::create_endpoint::{
    CreateEndpoint, CreateEndpointHandler,
};
use pigeon_application::commands::create_event_type::{
    CreateEventType, CreateEventTypeHandler,
};
use pigeon_application::commands::create_organization::{
    CreateOrganization, CreateOrganizationHandler,
};
use pigeon_application::commands::delete_application::{
    DeleteApplication, DeleteApplicationHandler,
};
use pigeon_application::commands::delete_endpoint::{
    DeleteEndpoint, DeleteEndpointHandler,
};
use pigeon_application::commands::delete_event_type::{
    DeleteEventType, DeleteEventTypeHandler,
};
use pigeon_application::commands::delete_organization::{
    DeleteOrganization, DeleteOrganizationHandler,
};
use pigeon_application::commands::create_oidc_config::{
    CreateOidcConfig, CreateOidcConfigHandler,
};
use pigeon_application::commands::delete_oidc_config::{
    DeleteOidcConfig, DeleteOidcConfigHandler,
};
use pigeon_application::commands::disable_endpoint::{
    DisableEndpoint, DisableEndpointHandler,
};
use pigeon_application::commands::replay_dead_letter::{
    ReplayDeadLetter, ReplayDeadLetterHandler,
};
use pigeon_application::commands::retry_attempt::{
    RetryAttempt, RetryAttemptHandler,
};
use pigeon_application::commands::retrigger_message::{
    RetriggerMessage, RetriggerMessageHandler, RetriggerMessageResult,
};
use pigeon_application::commands::send_message::{
    SendMessage, SendMessageHandler, SendMessageResult,
};
use pigeon_application::commands::send_test_event::{
    SendTestEvent, SendTestEventHandler, SendTestEventResult,
};
use pigeon_application::ports::stores::{EventTypeReadStore, MockMessageReadStore, MockAttemptReadStore as MockAttemptReadStoreImport, MockDeadLetterReadStore, MockOidcConfigReadStore};
use pigeon_application::ports::message_status::MessageWithStatus;
use pigeon_application::ports::stats_read_store::{AppStats, MockStatsReadStore};
use pigeon_application::ports::endpoint_stats_read_store::{EndpointStats, MockEndpointStatsReadStore};
use pigeon_application::ports::event_type_stats_read_store::{EventTypeStats, MockEventTypeStatsReadStore};
use pigeon_application::ports::audit_read_store::{AuditLogEntry, MockAuditReadStore};
use pigeon_application::queries::get_organization_by_id::{GetOrganizationById, GetOrganizationByIdHandler};
use pigeon_application::queries::list_organizations::{ListOrganizations, ListOrganizationsHandler};
use pigeon_application::queries::get_event_type_by_id::{GetEventTypeById, GetEventTypeByIdHandler};
use pigeon_application::queries::list_event_types_by_app::{ListEventTypesByApp, ListEventTypesByAppHandler};
use pigeon_application::queries::get_endpoint_by_id::{GetEndpointById, GetEndpointByIdHandler};
use pigeon_application::queries::list_endpoints_by_app::{ListEndpointsByApp, ListEndpointsByAppHandler};
use pigeon_application::queries::get_oidc_config_by_id::{GetOidcConfigById, GetOidcConfigByIdHandler};
use pigeon_application::queries::list_oidc_configs_by_org::{ListOidcConfigsByOrg, ListOidcConfigsByOrgHandler};
use pigeon_application::queries::get_message_by_id::{GetMessageById, GetMessageByIdHandler};
use pigeon_application::queries::list_messages_by_app::{ListMessagesByApp, ListMessagesByAppHandler};
use pigeon_application::queries::get_dead_letter_by_id::{GetDeadLetterById, GetDeadLetterByIdHandler};
use pigeon_application::queries::list_dead_letters_by_app::{ListDeadLettersByApp, ListDeadLettersByAppHandler};
use pigeon_application::queries::list_attempts_by_message::{ListAttemptsByMessage, ListAttemptsByMessageHandler};
use pigeon_application::queries::list_audit_log::{ListAuditLog, ListAuditLogHandler};
use pigeon_application::queries::get_app_stats::{GetAppStats, GetAppStatsHandler};
use pigeon_application::queries::get_endpoint_stats::{GetEndpointStats, GetEndpointStatsHandler};
use pigeon_application::queries::get_event_type_stats::{GetEventTypeStats, GetEventTypeStatsHandler};
use pigeon_application::commands::update_application::{
    UpdateApplication, UpdateApplicationHandler,
};
use pigeon_application::commands::update_endpoint::{
    UpdateEndpoint, UpdateEndpointHandler,
};
use pigeon_application::commands::update_event_type::{
    UpdateEventType, UpdateEventTypeHandler,
};
use pigeon_application::commands::update_organization::{
    UpdateOrganization, UpdateOrganizationHandler,
};
use pigeon_application::error::ApplicationError;
use pigeon_application::mediator::handler::{CommandHandler, QueryHandler};
use pigeon_application::queries::get_application_by_id::{
    GetApplicationById, GetApplicationByIdHandler,
};
use pigeon_application::queries::list_applications::{
    ListApplications, ListApplicationsHandler,
};
use pigeon_application::queries::PaginatedResult;
use pigeon_application::mediator::pipeline::RequestContext;
use pigeon_application::ports::unit_of_work::UnitOfWorkFactory;
use pigeon_application::test_support::fakes::{
    FakeApplicationReadStore, FakeEndpointReadStore, FakeEventTypeReadStore,
    FakeOrganizationReadStore, FakeUnitOfWorkFactory, OperationLog,
    SharedApplicationData, SharedAttemptData, SharedDeadLetterData, SharedEndpointData,
    SharedEventTypeData, SharedMessageData, SharedOidcConfigData, SharedOrganizationData,
};
use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::attempt::{Attempt, AttemptState, AttemptStatus};
use pigeon_domain::dead_letter::{DeadLetter, DeadLetterId, DeadLetterState};
use pigeon_domain::endpoint::{Endpoint, EndpointId, EndpointState};
use pigeon_domain::event_type::{EventType, EventTypeId, TEST_EVENT_TYPE_NAME};
use pigeon_domain::message::{Message, MessageId};
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId};
use pigeon_domain::organization::{Organization, OrganizationId};
use pigeon_domain::version::Version;
use serde_json::json;

#[derive(Debug, Default, World)]
pub struct AppWorld {
    // Create
    command: Option<CreateApplication>,
    result: Option<Result<Application, ApplicationError>>,
    log: Option<OperationLog>,

    // Org context for application commands
    org_id: Option<OrganizationId>,

    // Shared state for update/delete
    existing_app: Option<Application>,
    app_data: Option<SharedApplicationData>,

    // Update
    update_result: Option<Result<Application, ApplicationError>>,

    // Delete
    delete_result: Option<Result<(), ApplicationError>>,

    // Query: get by id
    get_result: Option<Result<Option<Application>, ApplicationError>>,

    // Query: list
    list_result: Option<Result<PaginatedResult<Application>, ApplicationError>>,

    // Event type
    create_event_type_command: Option<CreateEventType>,
    create_event_type_result: Option<Result<EventType, ApplicationError>>,
    existing_event_type: Option<EventType>,
    et_data: Option<SharedEventTypeData>,
    update_event_type_result: Option<Result<EventType, ApplicationError>>,
    delete_event_type_result: Option<Result<(), ApplicationError>>,

    // Endpoint
    create_endpoint_command: Option<CreateEndpoint>,
    create_endpoint_result: Option<Result<Endpoint, ApplicationError>>,
    existing_endpoint: Option<Endpoint>,
    ep_data: Option<SharedEndpointData>,
    update_endpoint_result: Option<Result<Endpoint, ApplicationError>>,
    delete_endpoint_result: Option<Result<(), ApplicationError>>,

    // SendMessage
    send_message_result: Option<Result<SendMessageResult, ApplicationError>>,
    endpoints: Option<Vec<Endpoint>>,
    event_type_id: Option<EventTypeId>,
    app_id: Option<ApplicationId>,
    msg_data: Option<SharedMessageData>,

    // Organization
    create_org_name: Option<String>,
    create_org_slug: Option<String>,
    create_org_oidc_issuer: Option<String>,
    create_org_oidc_audience: Option<String>,
    create_org_result: Option<Result<Organization, ApplicationError>>,
    create_org_oidc_data: Option<SharedOidcConfigData>,
    existing_org: Option<Organization>,
    org_data: Option<SharedOrganizationData>,
    update_org_result: Option<Result<Organization, ApplicationError>>,
    delete_org_result: Option<Result<(), ApplicationError>>,

    // Retrigger Message
    retrigger_result: Option<Result<RetriggerMessageResult, ApplicationError>>,
    existing_attempts: Option<Vec<pigeon_domain::attempt::Attempt>>,

    // OIDC Config
    create_oidc_config_result: Option<Result<OidcConfig, ApplicationError>>,
    existing_oidc_config: Option<OidcConfig>,
    oidc_data: Option<SharedOidcConfigData>,
    delete_oidc_config_result: Option<Result<(), ApplicationError>>,

    // Replay Dead Letter
    replay_dead_letter_result: Option<Result<DeadLetter, ApplicationError>>,
    existing_dead_letter: Option<DeadLetter>,
    dl_data: Option<SharedDeadLetterData>,

    // Retry Attempt
    retry_attempt_result: Option<Result<Attempt, ApplicationError>>,
    existing_attempt: Option<Attempt>,
    att_data: Option<SharedAttemptData>,

    // Send Test Event
    send_test_event_result: Option<Result<SendTestEventResult, ApplicationError>>,

    // Disable Endpoint
    disable_endpoint_result: Option<Result<(), ApplicationError>>,

    // Query: get/list organizations
    get_org_query_result: Option<Result<Option<Organization>, ApplicationError>>,
    list_orgs_result: Option<Result<PaginatedResult<Organization>, ApplicationError>>,

    // Query: get/list event types
    get_event_type_query_result: Option<Result<Option<EventType>, ApplicationError>>,
    list_event_types_result: Option<Result<PaginatedResult<EventType>, ApplicationError>>,

    // Query: get/list endpoints
    get_endpoint_query_result: Option<Result<Option<Endpoint>, ApplicationError>>,
    list_endpoints_result: Option<Result<PaginatedResult<Endpoint>, ApplicationError>>,

    // Query: get/list OIDC configs
    get_oidc_config_query_result: Option<Result<Option<OidcConfig>, ApplicationError>>,
    list_oidc_configs_result: Option<Result<PaginatedResult<OidcConfig>, ApplicationError>>,

    // Query: get/list messages
    get_message_query_result: Option<Result<Option<MessageWithStatus>, ApplicationError>>,
    list_messages_result: Option<Result<PaginatedResult<MessageWithStatus>, ApplicationError>>,

    // Query: get/list dead letters
    get_dead_letter_query_result: Option<Result<Option<DeadLetter>, ApplicationError>>,
    list_dead_letters_result: Option<Result<PaginatedResult<DeadLetter>, ApplicationError>>,

    // Query: list attempts
    list_attempts_result: Option<Result<Vec<Attempt>, ApplicationError>>,

    // Query: list audit log
    list_audit_log_result: Option<Result<PaginatedResult<AuditLogEntry>, ApplicationError>>,

    // Query: stats
    get_app_stats_result: Option<Result<AppStats, ApplicationError>>,
    get_endpoint_stats_result: Option<Result<EndpointStats, ApplicationError>>,
    get_event_type_stats_result: Option<Result<EventTypeStats, ApplicationError>>,
}

// ===== Create Application steps =====

#[given(regex = r#"a request to create an application named "([^"]*)" with uid "([^"]*)""#)]
async fn given_request(world: &mut AppWorld, name: String, uid: String) {
    let org_id = OrganizationId::new();
    world.org_id = Some(org_id.clone());
    world.command = Some(CreateApplication { org_id, name, uid });
    world.log = Some(OperationLog::new());
}

#[when("the create application command is executed")]
async fn when_executed(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
    let handler = CreateApplicationHandler::new();
    let command = world.command.take().unwrap();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.result = Some(handler.handle(command, &mut ctx).await);
}

#[then(regex = r#"the application should be created with name "([^"]*)""#)]
async fn then_created_with_name(world: &mut AppWorld, expected_name: String) {
    let app = world.result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(app.name(), expected_name);
}

#[then(regex = r#"the application should have uid "([^"]*)""#)]
async fn then_has_uid(world: &mut AppWorld, expected_uid: String) {
    let app = world.result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(app.uid(), expected_uid);
}

#[then("the application should have a generated id")]
async fn then_has_generated_id(world: &mut AppWorld) {
    let app = world.result.as_ref().unwrap().as_ref().unwrap();
    assert!(!app.id().as_uuid().is_nil());
}

#[then("the application store should contain the application")]
async fn then_store_contains(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log.entries().contains(&"application_store:insert".to_string()));
}

#[then("the command should fail with a validation error")]
async fn then_validation_error(world: &mut AppWorld) {
    let result = world.result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

// ===== Update Application steps =====

#[given(regex = r#"an existing application named "([^"]*)" with uid "([^"]*)""#)]
async fn given_existing_app(world: &mut AppWorld, name: String, uid: String) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let org_id = OrganizationId::new();
    let app = Application::new(org_id.clone(), name, uid).unwrap();

    // Seed the app into shared data
    {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.org_id = Some(org_id);
    world.existing_app = Some(app);
    world.app_data = Some(factory.app_data().clone());
    world.log = Some(log);
}

#[when(regex = r#"the update application command is executed with name "([^"]*)""#)]
async fn when_update_executed(world: &mut AppWorld, name: String) {
    let log = OperationLog::new();
    let app = world.existing_app.as_ref().unwrap();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_data(
        log.clone(),
        world.app_data.as_ref().unwrap().clone(),
    ));
    let handler = UpdateApplicationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), org_id.clone());
    ctx.set_uow(uow);

    world.update_result = Some(
        handler
            .handle(UpdateApplication {
                org_id,
                id: app.id().clone(),
                name,
                version: app.version(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the update application command is executed with a stale version")]
async fn when_update_stale_version(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app = world.existing_app.as_ref().unwrap();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_data(
        log.clone(),
        world.app_data.as_ref().unwrap().clone(),
    ));
    let handler = UpdateApplicationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), org_id.clone());
    ctx.set_uow(uow);

    world.update_result = Some(
        handler
            .handle(UpdateApplication {
                org_id,
                id: app.id().clone(),
                name: "new-name".into(),
                version: Version::new(999),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the update application command is executed for a non-existent application")]
async fn when_update_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = UpdateApplicationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_result = Some(
        handler
            .handle(UpdateApplication {
                org_id: OrganizationId::new(),
                id: ApplicationId::new(),
                name: "new-name".into(),
                version: Version::new(0),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then(regex = r#"the application should be updated with name "([^"]*)""#)]
async fn then_updated_with_name(world: &mut AppWorld, expected_name: String) {
    let app = world.update_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(app.name(), expected_name);
}

#[then("the application store should have saved the application")]
async fn then_store_saved(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log.entries().contains(&"application_store:save".to_string()));
}

#[then("the update command should fail with a validation error")]
async fn then_update_validation_error(world: &mut AppWorld) {
    let result = world.update_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[then("the update command should fail with a conflict error")]
async fn then_update_conflict_error(world: &mut AppWorld) {
    let result = world.update_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Conflict)),
        "expected Conflict error, got: {:?}",
        result
    );
}

#[then("the update command should fail with a not found error")]
async fn then_update_not_found_error(world: &mut AppWorld) {
    let result = world.update_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Delete Application steps =====

#[given(regex = r#"an application named "([^"]*)" with uid "([^"]*)" exists"#)]
async fn given_app_exists(world: &mut AppWorld, name: String, uid: String) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let org_id = OrganizationId::new();
    let app = Application::new(org_id.clone(), name, uid).unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.org_id = Some(org_id);
    world.existing_app = Some(app);
    world.app_data = Some(factory.app_data().clone());
    world.log = Some(log);
}

#[when("the delete application command is executed")]
async fn when_delete_executed(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app = world.existing_app.as_ref().unwrap();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_data(
        log.clone(),
        world.app_data.as_ref().unwrap().clone(),
    ));
    let handler = DeleteApplicationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_result = Some(
        handler
            .handle(DeleteApplication {
                org_id,
                id: app.id().clone(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the delete application command is executed for a non-existent application")]
async fn when_delete_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteApplicationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_result = Some(
        handler
            .handle(DeleteApplication {
                org_id: OrganizationId::new(),
                id: ApplicationId::new(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then("the application should be deleted successfully")]
async fn then_deleted_successfully(world: &mut AppWorld) {
    assert!(world.delete_result.as_ref().unwrap().is_ok());
}

#[then("the application store should have deleted the application")]
async fn then_store_deleted(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log.entries().contains(&"application_store:delete".to_string()));
}

#[then("the delete command should fail with a not found error")]
async fn then_delete_not_found_error(world: &mut AppWorld) {
    let result = world.delete_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Get Application By ID steps =====

#[given(regex = r#"an application named "([^"]*)" with uid "([^"]*)" has been created"#)]
async fn given_app_created(world: &mut AppWorld, name: String, uid: String) {
    let log = OperationLog::new();
    let data = SharedApplicationData::default();
    let org_id = OrganizationId::new();
    let app = Application::new(org_id.clone(), name, uid).unwrap();

    data.applications.lock().unwrap().push(app.clone());

    world.org_id = Some(org_id);
    world.existing_app = Some(app);
    world.app_data = Some(data);
    world.log = Some(log);
}

#[when("the get application by id query is executed")]
async fn when_get_by_id(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app = world.existing_app.as_ref().unwrap();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let read_store = Arc::new(FakeApplicationReadStore::new(
        log.clone(),
        world.app_data.as_ref().unwrap().clone(),
    ));
    let handler = GetApplicationByIdHandler::new(read_store);

    world.get_result = Some(
        handler
            .handle(GetApplicationById {
                org_id,
                id: app.id().clone(),
            })
            .await,
    );
    world.log = Some(log);
}

#[when("the get application by id query is executed for a non-existent id")]
async fn when_get_by_id_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let data = SharedApplicationData::default();
    let read_store = Arc::new(FakeApplicationReadStore::new(log.clone(), data));
    let handler = GetApplicationByIdHandler::new(read_store);

    world.get_result = Some(
        handler
            .handle(GetApplicationById {
                org_id: OrganizationId::new(),
                id: ApplicationId::new(),
            })
            .await,
    );
    world.log = Some(log);
}

#[then(regex = r#"the query should return the application with name "([^"]*)""#)]
async fn then_query_returns_app(world: &mut AppWorld, expected_name: String) {
    let app = world
        .get_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!(app.name(), expected_name);
}

#[then("the query should return no application")]
async fn then_query_returns_none(world: &mut AppWorld) {
    let result = world
        .get_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert!(result.is_none());
}

// ===== List Applications steps =====

#[given("the following applications exist:")]
async fn given_applications_exist(world: &mut AppWorld, step: &cucumber::gherkin::Step) {
    let log = OperationLog::new();
    let data = SharedApplicationData::default();
    let org_id = OrganizationId::new();

    if let Some(table) = &step.table {
        for row in table.rows.iter().skip(1) {
            let name = row[0].clone();
            let uid = row[1].clone();
            let app = Application::new(org_id.clone(), name, uid).unwrap();
            data.applications.lock().unwrap().push(app);
        }
    }

    world.org_id = Some(org_id);
    world.app_data = Some(data);
    world.log = Some(log);
}

#[when(regex = r"the list applications query is executed with offset (\d+) and limit (\d+)")]
async fn when_list(world: &mut AppWorld, offset: u64, limit: u64) {
    let log = OperationLog::new();
    let data = world
        .app_data
        .clone()
        .unwrap_or_default();
    let org_id = world.org_id.clone().unwrap_or_default();
    let read_store = Arc::new(FakeApplicationReadStore::new(log.clone(), data));
    let handler = ListApplicationsHandler::new(read_store);

    world.list_result = Some(
        handler
            .handle(ListApplications { org_id, search: None, offset, limit })
            .await,
    );
    world.log = Some(log);
}

#[then(regex = r"the result should contain (\d+) items")]
async fn then_result_contains_items(world: &mut AppWorld, count: usize) {
    let result = world.list_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
}

#[then(regex = r"the total count should be (\d+)")]
async fn then_total_count(world: &mut AppWorld, total: u64) {
    let result = world.list_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.total, total);
}

// ===== Create Event Type steps =====

#[given(regex = r#"a request to create an event type named "([^"]*)" for an application"#)]
async fn given_create_et_request(world: &mut AppWorld, name: String) {
    world.create_event_type_command = Some(CreateEventType {
        org_id: OrganizationId::new(),
        app_id: ApplicationId::new(),
        name,
        schema: None,
    });
    world.log = Some(OperationLog::new());
}

#[when("the create event type command is executed")]
async fn when_create_et_executed(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let et_data = world.et_data.take().unwrap_or_default();
    let et_store = Arc::new(FakeEventTypeReadStore::new(log.clone(), et_data));
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
    let handler = CreateEventTypeHandler::new(et_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    let command = world.create_event_type_command.take().unwrap();
    world.create_event_type_result = Some(handler.handle(command, &mut ctx).await);
}

#[then(regex = r#"the event type should be created with name "([^"]*)""#)]
async fn then_et_created_with_name(world: &mut AppWorld, expected_name: String) {
    let et = world
        .create_event_type_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!(et.name(), expected_name);
}

#[then("the event type should have a generated id")]
async fn then_et_has_generated_id(world: &mut AppWorld) {
    let et = world
        .create_event_type_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert!(!et.id().as_uuid().is_nil());
}

#[then("the event type store should contain the event type")]
async fn then_et_store_contains(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"event_type_store:insert".to_string()));
}

#[then("the create event type command should fail with a validation error")]
async fn then_create_et_validation_error(world: &mut AppWorld) {
    let result = world.create_event_type_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[given(regex = r#"an event type named "([^"]*)" already exists for the application"#)]
async fn given_existing_et_for_dupe_check(world: &mut AppWorld, name: String) {
    let app_id = ApplicationId::new();
    let org_id = OrganizationId::new();
    let et = EventType::new(app_id.clone(), name, None).unwrap();
    let et_data = SharedEventTypeData::default();
    et_data.event_types.lock().unwrap().push(et);
    world.app_id = Some(app_id);
    world.org_id = Some(org_id);
    world.et_data = Some(et_data);
    world.log = Some(OperationLog::new());
}

#[when(regex = r#"the create event type command is executed with name "([^"]*)""#)]
async fn when_create_et_with_name(world: &mut AppWorld, name: String) {
    let log = world.log.as_ref().unwrap().clone();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let et_data = world.et_data.take().unwrap_or_default();
    let et_store = Arc::new(FakeEventTypeReadStore::new(log.clone(), et_data));
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
    let handler = CreateEventTypeHandler::new(et_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.create_event_type_result = Some(
        handler
            .handle(CreateEventType {
                org_id,
                app_id,
                name,
                schema: None,
            }, &mut ctx)
            .await,
    );
}

// ===== Update Event Type steps =====

#[given(regex = r#"an existing event type named "([^"]*)""#)]
async fn given_existing_et(world: &mut AppWorld, name: String) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let et = EventType::new(ApplicationId::new(), name, None).unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.event_type_store().insert(&et).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.existing_event_type = Some(et);
    world.et_data = Some(factory.event_type_data().clone());
    world.log = Some(log);
}

#[when(regex = r#"the update event type command is executed with name "([^"]*)""#)]
async fn when_update_et_executed(world: &mut AppWorld, name: String) {
    let log = OperationLog::new();
    let et = world.existing_event_type.as_ref().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_event_type_data(
        log.clone(),
        world.et_data.as_ref().unwrap().clone(),
    ));
    let handler = UpdateEventTypeHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_event_type_result = Some(
        handler
            .handle(UpdateEventType {
                org_id: OrganizationId::new(),
                id: et.id().clone(),
                name,
                schema: None,
                version: et.version(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the update event type command is executed with a stale version")]
async fn when_update_et_stale_version(world: &mut AppWorld) {
    let log = OperationLog::new();
    let et = world.existing_event_type.as_ref().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_event_type_data(
        log.clone(),
        world.et_data.as_ref().unwrap().clone(),
    ));
    let handler = UpdateEventTypeHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_event_type_result = Some(
        handler
            .handle(UpdateEventType {
                org_id: OrganizationId::new(),
                id: et.id().clone(),
                name: "new.event".into(),
                schema: None,
                version: Version::new(999),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then(regex = r#"the event type should be updated with name "([^"]*)""#)]
async fn then_et_updated_with_name(world: &mut AppWorld, expected_name: String) {
    let et = world
        .update_event_type_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!(et.name(), expected_name);
}

#[then("the event type store should have saved the event type")]
async fn then_et_store_saved(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"event_type_store:save".to_string()));
}

#[then("the update event type command should fail with a validation error")]
async fn then_update_et_validation_error(world: &mut AppWorld) {
    let result = world.update_event_type_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[then("the update event type command should fail with a conflict error")]
async fn then_update_et_conflict_error(world: &mut AppWorld) {
    let result = world.update_event_type_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Conflict)),
        "expected Conflict error, got: {:?}",
        result
    );
}

// ===== Delete Event Type steps =====

#[given(regex = r#"an event type named "([^"]*)" exists"#)]
async fn given_et_exists(world: &mut AppWorld, name: String) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let et = EventType::new(ApplicationId::new(), name, None).unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.event_type_store().insert(&et).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.existing_event_type = Some(et);
    world.et_data = Some(factory.event_type_data().clone());
    world.log = Some(log);
}

#[when("the delete event type command is executed")]
async fn when_delete_et_executed(world: &mut AppWorld) {
    let log = OperationLog::new();
    let et = world.existing_event_type.as_ref().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_event_type_data(
        log.clone(),
        world.et_data.as_ref().unwrap().clone(),
    ));
    let handler = DeleteEventTypeHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_event_type_result = Some(
        handler
            .handle(DeleteEventType {
                org_id: OrganizationId::new(),
                id: et.id().clone(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the delete event type command is executed for a non-existent event type")]
async fn when_delete_et_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteEventTypeHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_event_type_result = Some(
        handler
            .handle(DeleteEventType {
                org_id: OrganizationId::new(),
                id: EventTypeId::new(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then("the event type should be deleted successfully")]
async fn then_et_deleted_successfully(world: &mut AppWorld) {
    assert!(world.delete_event_type_result.as_ref().unwrap().is_ok());
}

#[then("the event type store should have deleted the event type")]
async fn then_et_store_deleted(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"event_type_store:delete".to_string()));
}

#[then("the delete event type command should fail with a not found error")]
async fn then_delete_et_not_found_error(world: &mut AppWorld) {
    let result = world.delete_event_type_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Create Endpoint steps =====

#[given(regex = r#"a request to create an endpoint with url "([^"]*)" for an application"#)]
async fn given_create_ep_request(world: &mut AppWorld, url: String) {
    let app_id = ApplicationId::new();
    let org_id = OrganizationId::new();
    let et = EventType::new(app_id.clone(), "test.event".into(), None).unwrap();
    let et_id = et.id().clone();
    let et_data = SharedEventTypeData::default();
    et_data.event_types.lock().unwrap().push(et);

    world.create_endpoint_command = Some(CreateEndpoint {
        org_id,
        app_id,
        name: None,
        url,
        event_type_ids: vec![et_id],
    });
    world.et_data = Some(et_data);
    world.log = Some(OperationLog::new());
}

#[given("a request to create an endpoint with a non-existent event type")]
async fn given_create_ep_with_nonexistent_et(world: &mut AppWorld) {
    world.create_endpoint_command = Some(CreateEndpoint {
        org_id: OrganizationId::new(),
        app_id: ApplicationId::new(),
        name: None,
        url: "https://example.com/webhook".into(),
        event_type_ids: vec![EventTypeId::new()], // random ID, won't exist
    });
    world.log = Some(OperationLog::new());
}

#[when("the create endpoint command is executed")]
async fn when_create_ep_executed(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let et_data = world.et_data.take().unwrap_or_default();
    let et_store = Arc::new(FakeEventTypeReadStore::new(log.clone(), et_data));
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
    let handler = CreateEndpointHandler::new(et_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    let command = world.create_endpoint_command.take().unwrap();
    world.create_endpoint_result = Some(handler.handle(command, &mut ctx).await);
}

#[then(regex = r#"the endpoint should be created with url "([^"]*)""#)]
async fn then_ep_created_with_url(world: &mut AppWorld, expected_url: String) {
    let ep = world
        .create_endpoint_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!(ep.url(), expected_url);
}

#[then("the endpoint should have a generated id")]
async fn then_ep_has_generated_id(world: &mut AppWorld) {
    let ep = world
        .create_endpoint_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert!(!ep.id().as_uuid().is_nil());
}

#[then("the endpoint store should contain the endpoint")]
async fn then_ep_store_contains(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"endpoint_store:insert".to_string()));
}

#[then("the create endpoint command should fail with a validation error")]
async fn then_create_ep_validation_error(world: &mut AppWorld) {
    let result = world.create_endpoint_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

// ===== Update Endpoint steps =====

#[given(regex = r#"an existing endpoint with url "([^"]*)""#)]
async fn given_existing_ep(world: &mut AppWorld, url: String) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let ep = Endpoint::new(
        ApplicationId::new(),
        None,
        url,
        vec![EventTypeId::new()],
    )
    .unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.endpoint_store().insert(&ep).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.existing_endpoint = Some(ep);
    world.ep_data = Some(factory.endpoint_data().clone());
    world.log = Some(log);
}

#[when(regex = r#"the update endpoint command is executed with url "([^"]*)""#)]
async fn when_update_ep_executed(world: &mut AppWorld, url: String) {
    let log = OperationLog::new();
    let ep = world.existing_endpoint.as_ref().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_endpoint_data(
        log.clone(),
        world.ep_data.as_ref().unwrap().clone(),
    ));
    let et_store: Arc<dyn EventTypeReadStore> = Arc::new(FakeEventTypeReadStore::new(log.clone(), SharedEventTypeData::default()));
    let handler = UpdateEndpointHandler::new(et_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_endpoint_result = Some(
        handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id: ep.id().clone(),
                url,
                event_type_ids: vec![],
                version: ep.version(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the update endpoint command is executed with a non-existent event type")]
async fn when_update_ep_with_nonexistent_et(world: &mut AppWorld) {
    let log = OperationLog::new();
    let ep = world.existing_endpoint.as_ref().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_endpoint_data(
        log.clone(),
        world.ep_data.as_ref().unwrap().clone(),
    ));
    let et_store: Arc<dyn EventTypeReadStore> = Arc::new(FakeEventTypeReadStore::new(log.clone(), SharedEventTypeData::default()));
    let handler = UpdateEndpointHandler::new(et_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_endpoint_result = Some(
        handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id: ep.id().clone(),
                url: "https://example.com/webhook".into(),
                event_type_ids: vec![EventTypeId::new()], // non-existent
                version: ep.version(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the update endpoint command is executed with a stale version")]
async fn when_update_ep_stale_version(world: &mut AppWorld) {
    let log = OperationLog::new();
    let ep = world.existing_endpoint.as_ref().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_endpoint_data(
        log.clone(),
        world.ep_data.as_ref().unwrap().clone(),
    ));
    let et_store: Arc<dyn EventTypeReadStore> = Arc::new(FakeEventTypeReadStore::new(log.clone(), SharedEventTypeData::default()));
    let handler = UpdateEndpointHandler::new(et_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_endpoint_result = Some(
        handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id: ep.id().clone(),
                url: "https://new.example.com/webhook".into(),
                event_type_ids: vec![],
                version: Version::new(999),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then(regex = r#"the endpoint should be updated with url "([^"]*)""#)]
async fn then_ep_updated_with_url(world: &mut AppWorld, expected_url: String) {
    let ep = world
        .update_endpoint_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!(ep.url(), expected_url);
}

#[then("the endpoint store should have saved the endpoint")]
async fn then_ep_store_saved(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"endpoint_store:save".to_string()));
}

#[when("the update endpoint command is executed for a non-existent endpoint")]
async fn when_update_ep_non_existent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let et_store: Arc<dyn EventTypeReadStore> = Arc::new(FakeEventTypeReadStore::new(log.clone(), SharedEventTypeData::default()));
    let handler = UpdateEndpointHandler::new(et_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_endpoint_result = Some(
        handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id: EndpointId::new(),
                url: "https://example.com/webhook".into(),
                event_type_ids: vec![],
                version: Version::new(0),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then("the update endpoint command should fail with a validation error")]
async fn then_update_ep_validation_error(world: &mut AppWorld) {
    let result = world.update_endpoint_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[then("the update endpoint command should fail with a not found error")]
async fn then_update_ep_not_found_error(world: &mut AppWorld) {
    let result = world.update_endpoint_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

#[then("the update endpoint command should fail with a conflict error")]
async fn then_update_ep_conflict_error(world: &mut AppWorld) {
    let result = world.update_endpoint_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Conflict)),
        "expected Conflict error, got: {:?}",
        result
    );
}

// ===== Delete Endpoint steps =====

#[given(regex = r#"an endpoint with url "([^"]*)" exists"#)]
async fn given_ep_exists(world: &mut AppWorld, url: String) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let ep = Endpoint::new(
        ApplicationId::new(),
        None,
        url,
        vec![EventTypeId::new()],
    )
    .unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.endpoint_store().insert(&ep).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.existing_endpoint = Some(ep);
    world.ep_data = Some(factory.endpoint_data().clone());
    world.log = Some(log);
}

#[when("the delete endpoint command is executed")]
async fn when_delete_ep_executed(world: &mut AppWorld) {
    let log = OperationLog::new();
    let ep = world.existing_endpoint.as_ref().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_endpoint_data(
        log.clone(),
        world.ep_data.as_ref().unwrap().clone(),
    ));
    let handler = DeleteEndpointHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_endpoint_result = Some(
        handler
            .handle(DeleteEndpoint {
                org_id: OrganizationId::new(),
                id: ep.id().clone(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the delete endpoint command is executed for a non-existent endpoint")]
async fn when_delete_ep_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteEndpointHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_endpoint_result = Some(
        handler
            .handle(DeleteEndpoint {
                org_id: OrganizationId::new(),
                id: EndpointId::new(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then("the endpoint should be deleted successfully")]
async fn then_ep_deleted_successfully(world: &mut AppWorld) {
    assert!(world.delete_endpoint_result.as_ref().unwrap().is_ok());
}

#[then("the endpoint store should have deleted the endpoint")]
async fn then_ep_store_deleted(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"endpoint_store:delete".to_string()));
}

#[then("the delete endpoint command should fail with a not found error")]
async fn then_delete_ep_not_found_error(world: &mut AppWorld) {
    let result = world.delete_endpoint_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Send Message steps =====

#[given(regex = r#"an application with two enabled endpoints for event type "([^"]*)""#)]
async fn given_app_with_endpoints(world: &mut AppWorld, _event_type_name: String) {
    let app_id = ApplicationId::new();
    let event_type_id = EventTypeId::new();

    let ep1 = Endpoint::new(
        app_id.clone(),
        None,
        "https://a.com/hook".into(),
        vec![event_type_id.clone()],
    )
    .unwrap();
    let ep2 = Endpoint::new(
        app_id.clone(),
        None,
        "https://b.com/hook".into(),
        vec![event_type_id.clone()],
    )
    .unwrap();

    world.app_id = Some(app_id);
    world.event_type_id = Some(event_type_id);
    world.endpoints = Some(vec![ep1, ep2]);
    world.log = Some(OperationLog::new());
    world.msg_data = Some(SharedMessageData::default());
}

#[given(regex = r#"an application with a previously sent message with idempotency key "([^"]*)""#)]
async fn given_app_with_existing_message(world: &mut AppWorld, key: String) {
    let app_id = ApplicationId::new();
    let event_type_id = EventTypeId::new();

    let existing = Message::new(
        app_id.clone(),
        event_type_id.clone(),
        json!({"data": true}),
        Some(key),
        chrono::Duration::hours(24),
    )
    .unwrap();

    let msg_data = SharedMessageData::default();
    msg_data.messages.lock().unwrap().push(existing);

    world.app_id = Some(app_id);
    world.event_type_id = Some(event_type_id);
    world.endpoints = Some(vec![]);
    world.log = Some(OperationLog::new());
    world.msg_data = Some(msg_data);
}

#[given(regex = r#"an application with no endpoints for event type "([^"]*)""#)]
async fn given_app_with_no_endpoints(world: &mut AppWorld, _event_type_name: String) {
    let app_id = ApplicationId::new();
    let event_type_id = EventTypeId::new();

    world.app_id = Some(app_id);
    world.event_type_id = Some(event_type_id);
    world.endpoints = Some(vec![]);
    world.log = Some(OperationLog::new());
    world.msg_data = Some(SharedMessageData::default());
}

#[when(regex = r#"the send message command is executed with event type "([^"]*)""#)]
async fn when_send_message_with_event_type(world: &mut AppWorld, _event_type_name: String) {
    let log = world.log.as_ref().unwrap().clone();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let event_type_id = world.event_type_id.as_ref().unwrap().clone();
    let endpoints = world.endpoints.take().unwrap();
    let msg_data = world.msg_data.as_ref().unwrap().clone();

    let factory = Arc::new(FakeUnitOfWorkFactory::new_with_messages(
        log.clone(),
        msg_data,
    ));
    let endpoint_store = Arc::new(FakeEndpointReadStore::new(log, endpoints));
    let handler = SendMessageHandler::new(
        endpoint_store,
        chrono::Duration::hours(24),
    );
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.send_message_result = Some(
        handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!({"data": true}),
                idempotency_key: None,
            }, &mut ctx)
            .await,
    );
}

#[when(regex = r#"the send message command is executed with idempotency key "([^"]*)""#)]
async fn when_send_message_with_idempotency_key(world: &mut AppWorld, key: String) {
    let log = world.log.as_ref().unwrap().clone();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let event_type_id = world.event_type_id.as_ref().unwrap().clone();
    let endpoints = world.endpoints.take().unwrap();
    let msg_data = world.msg_data.as_ref().unwrap().clone();

    let factory = Arc::new(FakeUnitOfWorkFactory::new_with_messages(
        log.clone(),
        msg_data,
    ));
    let endpoint_store = Arc::new(FakeEndpointReadStore::new(log, endpoints));
    let handler = SendMessageHandler::new(
        endpoint_store,
        chrono::Duration::hours(24),
    );
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.send_message_result = Some(
        handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!({"data": true}),
                idempotency_key: Some(key),
            }, &mut ctx)
            .await,
    );
}

#[then("the message should be created")]
async fn then_message_created(world: &mut AppWorld) {
    let result = world.send_message_result.as_ref().unwrap().as_ref().unwrap();
    assert!(!result.message.id().as_uuid().is_nil());
}

#[then(regex = r"(\d+) attempts? should be created")]
async fn then_attempts_created(world: &mut AppWorld, count: usize) {
    let result = world.send_message_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.attempts_created, count);
}

#[then("the message should not be a duplicate")]
async fn then_not_duplicate(world: &mut AppWorld) {
    let result = world.send_message_result.as_ref().unwrap().as_ref().unwrap();
    assert!(!result.was_duplicate);
}

#[then("the message should be a duplicate")]
async fn then_is_duplicate(world: &mut AppWorld) {
    let result = world.send_message_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.was_duplicate);
}

#[when("the send message command is executed with a non-object payload")]
async fn when_send_message_non_object_payload(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let event_type_id = world.event_type_id.as_ref().unwrap().clone();
    let endpoints = world.endpoints.take().unwrap_or_default();

    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let endpoint_store = Arc::new(FakeEndpointReadStore::new(log, endpoints));
    let handler = SendMessageHandler::new(
        endpoint_store,
        chrono::Duration::hours(24),
    );
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.send_message_result = Some(
        handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!("not an object"),
                idempotency_key: None,
            }, &mut ctx)
            .await,
    );
}

#[then("the send message command should fail with a validation error")]
async fn then_send_message_validation_error(world: &mut AppWorld) {
    let result = world.send_message_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

// ===== Create Organization steps =====

#[given(regex = r#"a request to create an organization named "([^"]*)" with slug "([^"]*)""#)]
async fn given_create_org_request(world: &mut AppWorld, name: String, slug: String) {
    world.create_org_name = Some(name);
    world.create_org_slug = Some(slug);
    world.log = Some(OperationLog::new());
}

#[given(regex = r#"OIDC config with issuer "([^"]*)" and audience "([^"]*)""#)]
async fn given_oidc_config_for_org(world: &mut AppWorld, issuer: String, audience: String) {
    world.create_org_oidc_issuer = Some(issuer);
    world.create_org_oidc_audience = Some(audience);
}

#[when("the create organization command is executed")]
async fn when_create_org_executed(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let org_data = world.org_data.take().unwrap_or_default();
    let org_store = Arc::new(FakeOrganizationReadStore::new(log.clone(), org_data));
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

    let issuer = world.create_org_oidc_issuer.take().unwrap_or_default();
    let audience = world.create_org_oidc_audience.take().unwrap_or_default();
    let jwks_url = if issuer.is_empty() {
        String::new()
    } else {
        format!("{issuer}/.well-known/jwks.json")
    };

    let command = CreateOrganization {
        name: world.create_org_name.take().unwrap(),
        slug: world.create_org_slug.take().unwrap(),
        oidc_issuer_url: issuer,
        oidc_audience: audience,
        oidc_jwks_url: jwks_url,
    };

    let handler = CreateOrganizationHandler::new(org_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.create_org_result = Some(handler.handle(command, &mut ctx).await);
    world.create_org_oidc_data = Some(factory.oidc_config_data().clone());
}

#[then(regex = r#"the organization should be created with name "([^"]*)""#)]
async fn then_org_created_with_name(world: &mut AppWorld, expected_name: String) {
    let org = world.create_org_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(org.name(), expected_name);
}

#[then(regex = r#"the organization should have slug "([^"]*)""#)]
async fn then_org_has_slug(world: &mut AppWorld, expected_slug: String) {
    let org = world.create_org_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(org.slug(), expected_slug);
}

#[then("the organization should have a generated id")]
async fn then_org_has_generated_id(world: &mut AppWorld) {
    let org = world.create_org_result.as_ref().unwrap().as_ref().unwrap();
    assert!(!org.id().as_uuid().is_nil());
}

#[then("the organization store should contain the organization")]
async fn then_org_store_contains(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"organization_store:insert".to_string()));
}

#[then("the OIDC config store should contain a config for the organization")]
async fn then_oidc_store_contains_config(world: &mut AppWorld) {
    let oidc_data = world.create_org_oidc_data.as_ref().unwrap();
    let org = world.create_org_result.as_ref().unwrap().as_ref().unwrap();
    let configs = oidc_data.oidc_configs.lock().unwrap();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].org_id(), org.id());
}

#[then("the create organization command should fail with a validation error")]
async fn then_create_org_validation_error(world: &mut AppWorld) {
    let result = world.create_org_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[given(regex = r#"an organization with slug "([^"]*)" already exists"#)]
async fn given_org_with_slug_exists(world: &mut AppWorld, slug: String) {
    let existing = Organization::new("existing-org".into(), slug).unwrap();
    let org_data = SharedOrganizationData::default();
    org_data.organizations.lock().unwrap().push(existing);
    world.org_data = Some(org_data);
    world.log = Some(OperationLog::new());
}

#[when(regex = r#"the create organization command is executed with slug "([^"]*)""#)]
async fn when_create_org_with_slug(world: &mut AppWorld, slug: String) {
    let log = world.log.as_ref().unwrap().clone();
    let org_data = world.org_data.take().unwrap_or_default();
    let org_store = Arc::new(FakeOrganizationReadStore::new(log.clone(), org_data));
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
    let handler = CreateOrganizationHandler::new(org_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.create_org_result = Some(
        handler
            .handle(CreateOrganization {
                name: "new-org".into(),
                slug,
                oidc_issuer_url: "https://auth.example.com".into(),
                oidc_audience: "pigeon-api".into(),
                oidc_jwks_url: "https://auth.example.com/.well-known/jwks.json".into(),
            }, &mut ctx)
            .await,
    );
}

// ===== Update Organization steps =====

#[given(regex = r#"an existing organization named "([^"]*)" with slug "([^"]*)""#)]
async fn given_existing_org(world: &mut AppWorld, name: String, slug: String) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let org = Organization::new(name, slug).unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.organization_store().insert(&org).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.existing_org = Some(org);
    world.org_data = Some(factory.organization_data().clone());
    world.log = Some(log);
}

#[when(regex = r#"the update organization command is executed with name "([^"]*)""#)]
async fn when_update_org_executed(world: &mut AppWorld, name: String) {
    let log = OperationLog::new();
    let org = world.existing_org.as_ref().unwrap();
    let org_data = world.org_data.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_organization_data(log.clone(), org_data));
    let handler = UpdateOrganizationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_org_result = Some(
        handler
            .handle(UpdateOrganization {
                id: org.id().clone(),
                name,
                version: org.version(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the update organization command is executed with a stale version")]
async fn when_update_org_stale_version(world: &mut AppWorld) {
    let log = OperationLog::new();
    let org = world.existing_org.as_ref().unwrap();
    let org_data = world.org_data.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_organization_data(
        log.clone(),
        org_data,
    ));
    let handler = UpdateOrganizationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_org_result = Some(
        handler
            .handle(UpdateOrganization {
                id: org.id().clone(),
                name: "new-name".into(),
                version: Version::new(999),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the update organization command is executed for a non-existent organization")]
async fn when_update_org_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = UpdateOrganizationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.update_org_result = Some(
        handler
            .handle(UpdateOrganization {
                id: OrganizationId::new(),
                name: "new-name".into(),
                version: Version::new(0),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then(regex = r#"the organization should be updated with name "([^"]*)""#)]
async fn then_org_updated_with_name(world: &mut AppWorld, expected_name: String) {
    let org = world.update_org_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(org.name(), expected_name);
}

#[then("the organization store should have saved the organization")]
async fn then_org_store_saved(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"organization_store:save".to_string()));
}

#[then("the update organization command should fail with a validation error")]
async fn then_update_org_validation_error(world: &mut AppWorld) {
    let result = world.update_org_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[then("the update organization command should fail with a conflict error")]
async fn then_update_org_conflict_error(world: &mut AppWorld) {
    let result = world.update_org_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Conflict)),
        "expected Conflict error, got: {:?}",
        result
    );
}

#[then("the update organization command should fail with a not found error")]
async fn then_update_org_not_found_error(world: &mut AppWorld) {
    let result = world.update_org_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Delete Organization steps =====

#[given(regex = r#"an organization named "([^"]*)" with slug "([^"]*)" exists"#)]
async fn given_org_exists(world: &mut AppWorld, name: String, slug: String) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let org = Organization::new(name, slug).unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.organization_store().insert(&org).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.existing_org = Some(org);
    world.org_data = Some(factory.organization_data().clone());
    world.log = Some(log);
}

#[when("the delete organization command is executed")]
async fn when_delete_org_executed(world: &mut AppWorld) {
    let log = OperationLog::new();
    let org = world.existing_org.as_ref().unwrap();
    let org_data = world.org_data.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_organization_data(
        log.clone(),
        org_data,
    ));
    let handler = DeleteOrganizationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_org_result = Some(
        handler
            .handle(DeleteOrganization {
                id: org.id().clone(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[when("the delete organization command is executed for a non-existent organization")]
async fn when_delete_org_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteOrganizationHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_org_result = Some(
        handler
            .handle(DeleteOrganization {
                id: OrganizationId::new(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then("the organization should be deleted successfully")]
async fn then_org_deleted_successfully(world: &mut AppWorld) {
    assert!(world.delete_org_result.as_ref().unwrap().is_ok());
}

#[then("the organization store should have deleted the organization")]
async fn then_org_store_deleted(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    assert!(log
        .entries()
        .contains(&"organization_store:delete".to_string()));
}

#[then("the delete organization command should fail with a not found error")]
async fn then_delete_org_not_found_error(world: &mut AppWorld) {
    let result = world.delete_org_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== OIDC Config steps =====

#[given("an organization exists")]
async fn given_org_exists_for_oidc(world: &mut AppWorld) {
    world.org_id = Some(OrganizationId::new());
    world.log = Some(OperationLog::new());
}

#[when(regex = r#"an OIDC config is created with issuer "([^"]*)" and audience "([^"]*)""#)]
async fn when_create_oidc_config(world: &mut AppWorld, issuer: String, audience: String) {
    let log = world.log.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
    let handler = CreateOidcConfigHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    let jwks_url = if issuer.is_empty() {
        String::new()
    } else {
        format!("{issuer}/.well-known/jwks.json")
    };

    world.create_oidc_config_result = Some(
        handler
            .handle(CreateOidcConfig {
                org_id,
                issuer_url: issuer,
                audience,
                jwks_url,
            }, &mut ctx)
            .await,
    );
}

#[then("the OIDC config should be created successfully")]
async fn then_oidc_config_created(world: &mut AppWorld) {
    assert!(
        world.create_oidc_config_result.as_ref().unwrap().is_ok(),
        "expected Ok, got: {:?}",
        world.create_oidc_config_result
    );
}

#[then("it should have the correct issuer and audience")]
async fn then_oidc_config_has_correct_fields(world: &mut AppWorld) {
    let config = world
        .create_oidc_config_result
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!(config.issuer_url(), "https://auth.example.com");
    assert_eq!(config.audience(), "pigeon-api");
}

#[then("the OIDC config creation should fail with a validation error")]
async fn then_oidc_config_validation_error(world: &mut AppWorld) {
    let result = world.create_oidc_config_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[given("an organization with multiple OIDC configs exists")]
async fn given_org_with_multiple_oidc_configs(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let org_id = OrganizationId::new();
    let config1 = OidcConfig::new(
        org_id.clone(),
        "https://auth.example.com".into(),
        "pigeon-api".into(),
        "https://auth.example.com/.well-known/jwks.json".into(),
    )
    .unwrap();
    let config2 = OidcConfig::new(
        org_id.clone(),
        "https://auth2.example.com".into(),
        "pigeon-api-2".into(),
        "https://auth2.example.com/.well-known/jwks.json".into(),
    )
    .unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config1).await.unwrap();
        uow.oidc_config_store().insert(&config2).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.org_id = Some(org_id);
    world.existing_oidc_config = Some(config1);
    world.oidc_data = Some(factory.oidc_config_data().clone());
    world.log = Some(log);
}

#[given("an organization with an OIDC config exists")]
async fn given_org_with_oidc_config(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = FakeUnitOfWorkFactory::new(log.clone());
    let org_id = OrganizationId::new();
    let config = OidcConfig::new(
        org_id.clone(),
        "https://auth.example.com".into(),
        "pigeon-api".into(),
        "https://auth.example.com/.well-known/jwks.json".into(),
    )
    .unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config).await.unwrap();
        uow.commit().await.unwrap();
    }

    world.org_id = Some(org_id);
    world.existing_oidc_config = Some(config);
    world.oidc_data = Some(factory.oidc_config_data().clone());
    world.log = Some(log);
}

#[when("the OIDC config is deleted")]
async fn when_delete_oidc_config(world: &mut AppWorld) {
    let log = OperationLog::new();
    let config = world.existing_oidc_config.as_ref().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_oidc_config_data(
        log.clone(),
        world.oidc_data.as_ref().unwrap().clone(),
    ));
    let handler = DeleteOidcConfigHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_oidc_config_result = Some(
        handler
            .handle(DeleteOidcConfig {
                id: config.id().clone(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then("the OIDC config should be removed")]
async fn then_oidc_config_removed(world: &mut AppWorld) {
    assert!(world.delete_oidc_config_result.as_ref().unwrap().is_ok());
}

#[then("the deletion should fail with a validation error")]
async fn then_oidc_deletion_validation_error(world: &mut AppWorld) {
    let result = world.delete_oidc_config_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[when("a non-existent OIDC config is deleted")]
async fn when_delete_nonexistent_oidc_config(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteOidcConfigHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.delete_oidc_config_result = Some(
        handler
            .handle(DeleteOidcConfig {
                id: OidcConfigId::new(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then("the deletion should fail with a not found error")]
async fn then_oidc_deletion_not_found(world: &mut AppWorld) {
    let result = world.delete_oidc_config_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Retrigger Message steps =====

#[given("a message exists with a matching enabled endpoint")]
async fn given_message_with_matching_endpoint(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app_id = ApplicationId::new();
    let event_type_id = EventTypeId::new();
    let org_id = OrganizationId::new();

    let msg = Message::new(
        app_id.clone(),
        event_type_id.clone(),
        serde_json::json!({"test": true}),
        Some("retrigger-key".into()),
        chrono::Duration::hours(24),
    )
    .unwrap();

    let ep = Endpoint::new(
        app_id.clone(),
        None,
        "https://ep.com/hook".into(),
        vec![event_type_id.clone()],
    )
    .unwrap();

    world.org_id = Some(org_id);
    world.app_id = Some(app_id);
    world.event_type_id = Some(event_type_id);
    world.endpoints = Some(vec![ep]);
    world.msg_data = Some(SharedMessageData::default());
    world.msg_data.as_ref().unwrap().messages.lock().unwrap().push(msg);
    world.log = Some(log);
}

#[given("a message exists with no matching endpoints")]
async fn given_message_with_no_matching_endpoints(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app_id = ApplicationId::new();
    let event_type_id = EventTypeId::new();
    let org_id = OrganizationId::new();

    let msg = Message::new(
        app_id.clone(),
        event_type_id.clone(),
        serde_json::json!({"test": true}),
        None,
        chrono::Duration::hours(24),
    )
    .unwrap();

    world.org_id = Some(org_id);
    world.app_id = Some(app_id);
    world.event_type_id = Some(event_type_id);
    world.endpoints = Some(vec![]);
    world.msg_data = Some(SharedMessageData::default());
    world.msg_data.as_ref().unwrap().messages.lock().unwrap().push(msg);
    world.log = Some(log);
}

#[given("a message exists with an endpoint that already has an attempt")]
async fn given_message_with_existing_attempt(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app_id = ApplicationId::new();
    let event_type_id = EventTypeId::new();
    let org_id = OrganizationId::new();

    let msg = Message::new(
        app_id.clone(),
        event_type_id.clone(),
        serde_json::json!({"test": true}),
        Some("retrigger-dedup-key".into()),
        chrono::Duration::hours(24),
    )
    .unwrap();

    let ep = Endpoint::new(
        app_id.clone(),
        None,
        "https://ep.com/hook".into(),
        vec![event_type_id.clone()],
    )
    .unwrap();

    // Store the endpoint ID so we can create a matching attempt
    let ep_id = ep.id().clone();
    let msg_id = msg.id().clone();

    // Create an existing attempt for this endpoint+message
    let existing_attempt = pigeon_domain::attempt::Attempt::reconstitute(
        pigeon_domain::attempt::AttemptState {
            id: pigeon_domain::attempt::AttemptId::new(),
            message_id: msg_id,
            endpoint_id: ep_id,
            status: pigeon_domain::attempt::AttemptStatus::Succeeded,
            response_code: Some(200),
            response_body: None,
            attempted_at: Some(chrono::Utc::now()),
            next_attempt_at: None,
            attempt_number: 1,
            duration_ms: Some(50),
            version: pigeon_domain::version::Version::new(0),
        },
    );

    world.org_id = Some(org_id);
    world.app_id = Some(app_id);
    world.event_type_id = Some(event_type_id);
    world.endpoints = Some(vec![ep]);
    world.msg_data = Some(SharedMessageData::default());
    world.msg_data.as_ref().unwrap().messages.lock().unwrap().push(msg);
    // Store the existing attempt so the "when" step can use it
    world.existing_attempts = Some(vec![existing_attempt]);
    world.log = Some(log);
}

#[when("the retrigger message command is executed")]
async fn when_retrigger_executed(world: &mut AppWorld) {
    use pigeon_application::ports::message_status::MessageWithStatus;
    use pigeon_application::ports::stores::MockAttemptReadStore;

    let log = world.log.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let endpoints = world.endpoints.take().unwrap_or_default();
    let messages = world.msg_data.as_ref().unwrap().messages.lock().unwrap().clone();
    let message_id = messages.first().unwrap().id().clone();
    let existing_attempts = world.existing_attempts.take().unwrap_or_default();

    let mut mock_msg_store = MockMessageReadStore::new();
    let msg_clone = messages.first().unwrap().clone();
    mock_msg_store
        .expect_find_by_id()
        .returning(move |_, _| Ok(Some(MessageWithStatus {
            message: msg_clone.clone(),
            attempts_created: 0,
            succeeded: 0,
            failed: 0,
            dead_lettered: 0,
        })));

    let mut mock_att_store = MockAttemptReadStore::new();
    mock_att_store
        .expect_list_by_message()
        .returning(move |_, _| Ok(existing_attempts.clone()));

    let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), endpoints));
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

    let handler = RetriggerMessageHandler::new(
        Arc::new(mock_msg_store),
        endpoint_store,
        Arc::new(mock_att_store),
    );
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.retrigger_result = Some(
        handler
            .handle(RetriggerMessage { message_id, org_id }, &mut ctx)
            .await,
    );
}

#[when("a non-existent message is retriggered")]
async fn when_retrigger_nonexistent(world: &mut AppWorld) {
    use pigeon_application::ports::stores::MockAttemptReadStore;

    let log = OperationLog::new();

    let mut mock_msg_store = MockMessageReadStore::new();
    mock_msg_store
        .expect_find_by_id()
        .returning(|_, _| Ok(None));

    let mut mock_att_store = MockAttemptReadStore::new();
    mock_att_store.expect_list_by_message().returning(|_, _| Ok(vec![]));

    let endpoint_store = Arc::new(FakeEndpointReadStore::new(log.clone(), vec![]));
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));

    let handler = RetriggerMessageHandler::new(
        Arc::new(mock_msg_store),
        endpoint_store,
        Arc::new(mock_att_store),
    );
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);

    world.retrigger_result = Some(
        handler
            .handle(RetriggerMessage {
                message_id: pigeon_domain::message::MessageId::new(),
                org_id: OrganizationId::new(),
            }, &mut ctx)
            .await,
    );
    world.log = Some(log);
}

#[then("new delivery attempts should be created")]
async fn then_retrigger_attempts_created(world: &mut AppWorld) {
    let result = world.retrigger_result.as_ref().unwrap();
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert!(result.as_ref().unwrap().attempts_created > 0);
}

#[then("the attempt count should match the number of matching endpoints")]
async fn then_retrigger_attempt_count_matches(world: &mut AppWorld) {
    let result = world.retrigger_result.as_ref().unwrap().as_ref().unwrap();
    // We set up 1 endpoint in the "matching" scenario
    assert_eq!(result.attempts_created, 1);
}

#[then("the retrigger should fail with a validation error")]
async fn then_retrigger_validation_error(world: &mut AppWorld) {
    let result = world.retrigger_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[then("the retrigger should fail with a not found error")]
async fn then_retrigger_not_found(world: &mut AppWorld) {
    let result = world.retrigger_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Replay Dead Letter steps =====

#[given("a dead letter exists that has not been replayed")]
async fn given_unreplayed_dead_letter(world: &mut AppWorld) {
    let log = OperationLog::new();
    let dl = pigeon_domain::test_support::any_dead_letter();
    let dl_data = SharedDeadLetterData::default();
    dl_data.dead_letters.lock().unwrap().push(dl.clone());
    world.existing_dead_letter = Some(dl);
    world.dl_data = Some(dl_data);
    world.log = Some(log);
}

#[given("a dead letter exists that has already been replayed")]
async fn given_replayed_dead_letter(world: &mut AppWorld) {
    let log = OperationLog::new();
    let mut state = DeadLetterState::fake();
    state.replayed_at = Some(chrono::Utc::now());
    let dl = DeadLetter::reconstitute(state);
    let dl_data = SharedDeadLetterData::default();
    dl_data.dead_letters.lock().unwrap().push(dl.clone());
    world.existing_dead_letter = Some(dl);
    world.dl_data = Some(dl_data);
    world.log = Some(log);
}

#[when("the replay dead letter command is executed")]
async fn when_replay_dead_letter(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let dl = world.existing_dead_letter.as_ref().unwrap();
    let dl_data = world.dl_data.take().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_dead_letter_data(log, dl_data));

    let handler = ReplayDeadLetterHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.replay_dead_letter_result = Some(
        handler
            .handle(ReplayDeadLetter {
                org_id: OrganizationId::new(),
                dead_letter_id: dl.id().clone(),
            }, &mut ctx)
            .await,
    );
}

#[when("a non-existent dead letter is replayed")]
async fn when_replay_nonexistent_dead_letter(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    world.log = Some(log);

    let handler = ReplayDeadLetterHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.replay_dead_letter_result = Some(
        handler
            .handle(ReplayDeadLetter {
                org_id: OrganizationId::new(),
                dead_letter_id: pigeon_domain::dead_letter::DeadLetterId::new(),
            }, &mut ctx)
            .await,
    );
}

#[then("the dead letter should be marked as replayed")]
async fn then_dead_letter_replayed(world: &mut AppWorld) {
    let result = world.replay_dead_letter_result.as_ref().unwrap();
    let dl = result.as_ref().expect("expected Ok result");
    assert!(dl.replayed_at().is_some(), "replayed_at should be set");
}

#[then("a new delivery attempt should be created")]
async fn then_replay_attempt_created(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    let entries = log.entries();
    assert!(
        entries.iter().any(|e| e == "attempt_store:insert"),
        "expected attempt_store:insert in log, got: {:?}",
        entries
    );
}

#[then("the replay should fail with a validation error")]
async fn then_replay_validation_error(world: &mut AppWorld) {
    let result = world.replay_dead_letter_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[then("the replay should fail with a not found error")]
async fn then_replay_not_found(world: &mut AppWorld) {
    let result = world.replay_dead_letter_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Retry Attempt steps =====

#[given("a failed delivery attempt exists")]
async fn given_failed_attempt(world: &mut AppWorld) {
    let log = OperationLog::new();
    let mut state = AttemptState::fake();
    state.status = AttemptStatus::Failed;
    state.next_attempt_at = None;
    let attempt = Attempt::reconstitute(state);
    let att_data = SharedAttemptData::default();
    att_data.attempts.lock().unwrap().push(attempt.clone());
    world.existing_attempt = Some(attempt);
    world.att_data = Some(att_data);
    world.log = Some(log);
}

#[given("a pending delivery attempt exists")]
async fn given_pending_attempt(world: &mut AppWorld) {
    let log = OperationLog::new();
    let state = AttemptState::fake(); // default is Pending
    let attempt = Attempt::reconstitute(state);
    let att_data = SharedAttemptData::default();
    att_data.attempts.lock().unwrap().push(attempt.clone());
    world.existing_attempt = Some(attempt);
    world.att_data = Some(att_data);
    world.log = Some(log);
}

#[when("the retry attempt command is executed")]
async fn when_retry_attempt(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let attempt = world.existing_attempt.as_ref().unwrap();
    let att_data = world.att_data.take().unwrap();
    let factory = Arc::new(FakeUnitOfWorkFactory::with_attempt_data(log, att_data));

    let handler = RetryAttemptHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.retry_attempt_result = Some(
        handler
            .handle(RetryAttempt {
                org_id: OrganizationId::new(),
                attempt_id: attempt.id().clone(),
            }, &mut ctx)
            .await,
    );
}

#[when("a non-existent attempt is retried")]
async fn when_retry_nonexistent_attempt(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    world.log = Some(log);

    let handler = RetryAttemptHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.retry_attempt_result = Some(
        handler
            .handle(RetryAttempt {
                org_id: OrganizationId::new(),
                attempt_id: pigeon_domain::attempt::AttemptId::new(),
            }, &mut ctx)
            .await,
    );
}

#[then("the attempt status should be pending")]
async fn then_attempt_is_pending(world: &mut AppWorld) {
    let result = world.retry_attempt_result.as_ref().unwrap();
    let attempt = result.as_ref().expect("expected Ok result");
    assert_eq!(attempt.status(), AttemptStatus::Pending);
}

#[then("the retry should fail with a validation error")]
async fn then_retry_validation_error(world: &mut AppWorld) {
    let result = world.retry_attempt_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Validation(_))),
        "expected Validation error, got: {:?}",
        result
    );
}

#[then("the retry should fail with a not found error")]
async fn then_retry_not_found(world: &mut AppWorld) {
    let result = world.retry_attempt_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Send Test Event steps =====

#[given("an application with the pigeon.test event type and an endpoint")]
async fn given_app_with_test_event_type(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app_id = ApplicationId::new();
    let org_id = OrganizationId::new();

    let et = EventType::new(app_id.clone(), TEST_EVENT_TYPE_NAME.to_string(), None).unwrap();
    let et_data = SharedEventTypeData::default();
    et_data.event_types.lock().unwrap().push(et.clone());

    let endpoint = Endpoint::new(
        app_id.clone(),
        None,
        "https://example.com/webhook".to_string(),
        vec![et.id().clone()],
    )
    .unwrap();

    world.app_id = Some(app_id);
    world.org_id = Some(org_id);
    world.event_type_id = Some(et.id().clone());
    world.et_data = Some(et_data);
    world.existing_endpoint = Some(endpoint);
    world.log = Some(log);
}

#[given("an application without the pigeon.test event type")]
async fn given_app_without_test_event_type(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app_id = ApplicationId::new();
    let org_id = OrganizationId::new();

    let et_data = SharedEventTypeData::default(); // empty — no event types

    let endpoint = Endpoint::new(
        app_id.clone(),
        None,
        "https://example.com/webhook".to_string(),
        vec![EventTypeId::new()],
    )
    .unwrap();

    world.app_id = Some(app_id);
    world.org_id = Some(org_id);
    world.et_data = Some(et_data);
    world.existing_endpoint = Some(endpoint);
    world.log = Some(log);
}

#[when("the send test event command is executed")]
async fn when_send_test_event(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let endpoint = world.existing_endpoint.as_ref().unwrap();
    let et_data = world.et_data.take().unwrap();

    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let et_read_store = Arc::new(FakeEventTypeReadStore::new(log, et_data));

    let handler = SendTestEventHandler::new(et_read_store);
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.send_test_event_result = Some(
        handler
            .handle(SendTestEvent {
                org_id,
                app_id,
                endpoint_id: endpoint.id().clone(),
            }, &mut ctx)
            .await,
    );
}

#[then("a test message should be created")]
async fn then_test_message_created(world: &mut AppWorld) {
    let result = world.send_test_event_result.as_ref().unwrap();
    let r = result.as_ref().expect("expected Ok result");
    assert!(!r.message.id().as_uuid().is_nil());
}

#[then("a delivery attempt should be created for the endpoint")]
async fn then_test_event_attempt_created(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    let entries = log.entries();
    assert!(
        entries.iter().any(|e| e == "attempt_store:insert"),
        "expected attempt_store:insert in log, got: {:?}",
        entries
    );
}

#[then("the send test event should fail with an internal error")]
async fn then_send_test_event_internal_error(world: &mut AppWorld) {
    let result = world.send_test_event_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::Internal(_))),
        "expected Internal error, got: {:?}",
        result
    );
}

// ===== Disable Endpoint steps =====

#[given("an enabled endpoint exists")]
async fn given_enabled_endpoint(world: &mut AppWorld) {
    let log = OperationLog::new();
    let app_id = ApplicationId::new();
    let ep = Endpoint::new(
        app_id.clone(),
        None,
        "https://example.com/webhook".to_string(),
        vec![EventTypeId::new()],
    )
    .unwrap();
    let ep_data = SharedEndpointData::default();
    ep_data.endpoints.lock().unwrap().push(ep.clone());

    world.app_id = Some(app_id);
    world.existing_endpoint = Some(ep);
    world.ep_data = Some(ep_data);
    world.log = Some(log);
}

#[given("a disabled endpoint exists")]
async fn given_disabled_endpoint(world: &mut AppWorld) {
    let log = OperationLog::new();
    let mut state = EndpointState::fake();
    state.enabled = false;
    let ep = Endpoint::reconstitute(state);
    let ep_data = SharedEndpointData::default();
    ep_data.endpoints.lock().unwrap().push(ep.clone());

    world.app_id = Some(ep.app_id().clone());
    world.existing_endpoint = Some(ep);
    world.ep_data = Some(ep_data);
    world.log = Some(log);
}

#[when("the disable endpoint command is executed")]
async fn when_disable_endpoint(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let ep = world.existing_endpoint.as_ref().unwrap();
    let ep_data = world.ep_data.take().unwrap();

    let factory = Arc::new(FakeUnitOfWorkFactory::with_endpoint_data(log, ep_data));

    let handler = DisableEndpointHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.disable_endpoint_result = Some(
        handler
            .handle(DisableEndpoint {
                app_id,
                endpoint_id: ep.id().clone(),
            }, &mut ctx)
            .await,
    );
}

#[when("a non-existent endpoint is disabled")]
async fn when_disable_nonexistent_endpoint(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    world.log = Some(log);

    let handler = DisableEndpointHandler::new();
    let uow = factory.begin().await.unwrap();
    let mut ctx = RequestContext::new("Test", "test".into(), OrganizationId::new());
    ctx.set_uow(uow);
    world.disable_endpoint_result = Some(
        handler
            .handle(DisableEndpoint {
                app_id: ApplicationId::new(),
                endpoint_id: EndpointId::new(),
            }, &mut ctx)
            .await,
    );
}

#[then("the endpoint should be disabled")]
async fn then_endpoint_disabled(world: &mut AppWorld) {
    let result = world.disable_endpoint_result.as_ref().unwrap();
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);

    let log = world.log.as_ref().unwrap();
    let entries = log.entries();
    assert!(
        entries.iter().any(|e| e == "endpoint_store:save"),
        "expected endpoint_store:save in log, got: {:?}",
        entries
    );
}

#[then("an endpoint updated event should be emitted")]
async fn then_endpoint_updated_event_emitted(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap();
    let entries = log.entries();
    assert!(
        entries.iter().any(|e| e == "uow:emit_event:endpoint_updated"),
        "expected uow:emit_event:endpoint_updated in log, got: {:?}",
        entries
    );
}

#[then("the command should succeed without changes")]
async fn then_disable_noop(world: &mut AppWorld) {
    let result = world.disable_endpoint_result.as_ref().unwrap();
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);

    let log = world.log.as_ref().unwrap();
    let entries = log.entries();
    assert!(
        !entries.iter().any(|e| e == "endpoint_store:save"),
        "expected no endpoint_store:save in log (no-op), got: {:?}",
        entries
    );
}

#[then("the disable should fail with a not found error")]
async fn then_disable_not_found(world: &mut AppWorld) {
    let result = world.disable_endpoint_result.as_ref().unwrap();
    assert!(
        matches!(result, Err(ApplicationError::NotFound)),
        "expected NotFound error, got: {:?}",
        result
    );
}

// ===== Get Organization by ID steps =====

#[given("an organization exists in the read store")]
async fn given_org_in_read_store(world: &mut AppWorld) {
    let org = pigeon_domain::test_support::any_organization();
    let data = SharedOrganizationData::default();
    data.organizations.lock().unwrap().push(org.clone());
    world.existing_org = Some(org);
    world.org_data = Some(data);
}

#[when("the get organization by id query is executed")]
async fn when_get_org_by_id(world: &mut AppWorld) {
    let org = world.existing_org.as_ref().unwrap();
    let data = world.org_data.as_ref().unwrap().clone();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeOrganizationReadStore::new(log, data));
    let handler = GetOrganizationByIdHandler::new(read_store);

    world.get_org_query_result = Some(
        handler
            .handle(GetOrganizationById {
                id: org.id().clone(),
            })
            .await,
    );
}

#[when("the get organization by id query is executed for a non-existent id")]
async fn when_get_org_by_id_nonexistent(world: &mut AppWorld) {
    let data = SharedOrganizationData::default();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeOrganizationReadStore::new(log, data));
    let handler = GetOrganizationByIdHandler::new(read_store);

    world.get_org_query_result = Some(
        handler
            .handle(GetOrganizationById {
                id: OrganizationId::new(),
            })
            .await,
    );
}

#[then("the organization should be returned")]
async fn then_org_returned(world: &mut AppWorld) {
    let result = world.get_org_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_some());
}

#[then("no organization should be returned")]
async fn then_no_org_returned(world: &mut AppWorld) {
    let result = world.get_org_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_none());
}

// ===== List Organizations steps =====

#[given("no organizations exist")]
async fn given_no_orgs(world: &mut AppWorld) {
    world.org_data = Some(SharedOrganizationData::default());
}

#[given(regex = r"(\d+) organizations exist")]
async fn given_n_orgs_exist(world: &mut AppWorld, count: usize) {
    let data = SharedOrganizationData::default();
    for i in 0..count {
        let org = Organization::new(format!("org-{i}"), format!("org-{i}")).unwrap();
        data.organizations.lock().unwrap().push(org);
    }
    world.org_data = Some(data);
}

#[when("the list organizations query is executed")]
async fn when_list_orgs(world: &mut AppWorld) {
    let data = world.org_data.as_ref().unwrap().clone();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeOrganizationReadStore::new(log, data));
    let handler = ListOrganizationsHandler::new(read_store);

    world.list_orgs_result = Some(
        handler
            .handle(ListOrganizations {
                offset: 0,
                limit: 100,
            })
            .await,
    );
}

#[when(regex = r"the list organizations query is executed with offset (\d+) and limit (\d+)")]
async fn when_list_orgs_paginated(world: &mut AppWorld, offset: u64, limit: u64) {
    let data = world.org_data.as_ref().unwrap().clone();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeOrganizationReadStore::new(log, data));
    let handler = ListOrganizationsHandler::new(read_store);

    world.list_orgs_result = Some(
        handler.handle(ListOrganizations { offset, limit }).await,
    );
}

#[then("the result should be an empty paginated list")]
async fn then_empty_paginated_list(world: &mut AppWorld) {
    let result = world.list_orgs_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.items.is_empty());
    assert_eq!(result.total, 0);
}

#[then(regex = r"the result should contain (\d+) organizations")]
async fn then_contains_n_orgs(world: &mut AppWorld, count: usize) {
    let result = world.list_orgs_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
}

#[then(regex = r"the result should contain (\d+) organization with total (\d+)")]
async fn then_contains_n_org_with_total(world: &mut AppWorld, count: usize, total: u64) {
    let result = world.list_orgs_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
    assert_eq!(result.total, total);
}

// ===== Get Event Type by ID steps =====

#[given("an event type exists in the read store")]
async fn given_et_in_read_store(world: &mut AppWorld) {
    let et = pigeon_domain::test_support::any_event_type();
    let data = SharedEventTypeData::default();
    data.event_types.lock().unwrap().push(et.clone());
    world.existing_event_type = Some(et);
    world.et_data = Some(data);
    world.org_id = Some(OrganizationId::new());
}

#[when("the get event type by id query is executed")]
async fn when_get_et_by_id(world: &mut AppWorld) {
    let et = world.existing_event_type.as_ref().unwrap();
    let data = world.et_data.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeEventTypeReadStore::new(log, data));
    let handler = GetEventTypeByIdHandler::new(read_store);

    world.get_event_type_query_result = Some(
        handler
            .handle(GetEventTypeById {
                id: et.id().clone(),
                org_id,
            })
            .await,
    );
}

#[when("the get event type by id query is executed for a non-existent id")]
async fn when_get_et_by_id_nonexistent(world: &mut AppWorld) {
    let data = SharedEventTypeData::default();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeEventTypeReadStore::new(log, data));
    let handler = GetEventTypeByIdHandler::new(read_store);

    world.get_event_type_query_result = Some(
        handler
            .handle(GetEventTypeById {
                id: EventTypeId::new(),
                org_id: OrganizationId::new(),
            })
            .await,
    );
}

#[then("the event type should be returned")]
async fn then_et_returned(world: &mut AppWorld) {
    let result = world.get_event_type_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_some());
}

#[then("no event type should be returned")]
async fn then_no_et_returned(world: &mut AppWorld) {
    let result = world.get_event_type_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_none());
}

// ===== List Event Types by App steps =====

#[given("an application with no event types")]
async fn given_app_no_event_types(world: &mut AppWorld) {
    world.app_id = Some(ApplicationId::new());
    world.org_id = Some(OrganizationId::new());
    world.et_data = Some(SharedEventTypeData::default());
}

#[given(regex = r"an application with (\d+) event types")]
async fn given_app_with_n_event_types(world: &mut AppWorld, count: usize) {
    let app_id = ApplicationId::new();
    let data = SharedEventTypeData::default();
    for i in 0..count {
        let et = EventType::new(app_id.clone(), format!("event.type.{i}"), None).unwrap();
        data.event_types.lock().unwrap().push(et);
    }
    world.app_id = Some(app_id);
    world.org_id = Some(OrganizationId::new());
    world.et_data = Some(data);
}

#[when("the list event types query is executed")]
async fn when_list_event_types(world: &mut AppWorld) {
    let data = world.et_data.as_ref().unwrap().clone();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeEventTypeReadStore::new(log, data));
    let handler = ListEventTypesByAppHandler::new(read_store);

    world.list_event_types_result = Some(
        handler
            .handle(ListEventTypesByApp {
                app_id,
                org_id,
                offset: 0,
                limit: 100,
            })
            .await,
    );
}

#[when(regex = r"the list event types query is executed with offset (\d+) and limit (\d+)")]
async fn when_list_event_types_paginated(world: &mut AppWorld, offset: u64, limit: u64) {
    let data = world.et_data.as_ref().unwrap().clone();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeEventTypeReadStore::new(log, data));
    let handler = ListEventTypesByAppHandler::new(read_store);

    world.list_event_types_result = Some(
        handler
            .handle(ListEventTypesByApp {
                app_id,
                org_id,
                offset,
                limit,
            })
            .await,
    );
}

#[then("the result should be an empty event type list")]
async fn then_empty_et_list(world: &mut AppWorld) {
    let result = world.list_event_types_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.items.is_empty());
    assert_eq!(result.total, 0);
}

#[then(regex = r"the result should contain (\d+) event types")]
async fn then_contains_n_event_types(world: &mut AppWorld, count: usize) {
    let result = world.list_event_types_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
}

#[then(regex = r"the result should contain (\d+) event type with total (\d+)")]
async fn then_contains_n_et_with_total(world: &mut AppWorld, count: usize, total: u64) {
    let result = world.list_event_types_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
    assert_eq!(result.total, total);
}

// ===== Get Endpoint by ID steps =====

#[given("an endpoint exists in the read store")]
async fn given_ep_in_read_store(world: &mut AppWorld) {
    let ep = pigeon_domain::test_support::any_endpoint();
    world.existing_endpoint = Some(ep);
    world.org_id = Some(OrganizationId::new());
}

#[when("the get endpoint by id query is executed")]
async fn when_get_ep_by_id(world: &mut AppWorld) {
    let ep = world.existing_endpoint.as_ref().unwrap();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeEndpointReadStore::new(log, vec![ep.clone()]));
    let handler = GetEndpointByIdHandler::new(read_store);

    world.get_endpoint_query_result = Some(
        handler
            .handle(GetEndpointById {
                id: ep.id().clone(),
                org_id,
            })
            .await,
    );
}

#[when("the get endpoint by id query is executed for a non-existent id")]
async fn when_get_ep_by_id_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let read_store = Arc::new(FakeEndpointReadStore::new(log, vec![]));
    let handler = GetEndpointByIdHandler::new(read_store);

    world.get_endpoint_query_result = Some(
        handler
            .handle(GetEndpointById {
                id: EndpointId::new(),
                org_id: OrganizationId::new(),
            })
            .await,
    );
}

#[then("the endpoint should be returned")]
async fn then_ep_returned(world: &mut AppWorld) {
    let result = world.get_endpoint_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_some());
}

#[then("no endpoint should be returned")]
async fn then_no_ep_returned(world: &mut AppWorld) {
    let result = world.get_endpoint_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_none());
}

// ===== List Endpoints by App steps =====

#[given("an application with no endpoints")]
async fn given_app_no_endpoints(world: &mut AppWorld) {
    world.app_id = Some(ApplicationId::new());
    world.org_id = Some(OrganizationId::new());
    world.endpoints = Some(vec![]);
}

#[given(regex = r"an application with (\d+) endpoints")]
async fn given_app_with_n_endpoints(world: &mut AppWorld, count: usize) {
    let app_id = ApplicationId::new();
    let mut eps = Vec::new();
    for i in 0..count {
        let ep = Endpoint::new(
            app_id.clone(),
            None,
            format!("https://ep{i}.example.com/webhook"),
            vec![EventTypeId::new()],
        )
        .unwrap();
        eps.push(ep);
    }
    world.app_id = Some(app_id);
    world.org_id = Some(OrganizationId::new());
    world.endpoints = Some(eps);
}

#[when("the list endpoints query is executed")]
async fn when_list_endpoints(world: &mut AppWorld) {
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let endpoints = world.endpoints.take().unwrap_or_default();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeEndpointReadStore::new(log, endpoints));
    let handler = ListEndpointsByAppHandler::new(read_store);

    world.list_endpoints_result = Some(
        handler
            .handle(ListEndpointsByApp {
                app_id,
                org_id,
                offset: 0,
                limit: 100,
            })
            .await,
    );
}

#[when(regex = r"the list endpoints query is executed with offset (\d+) and limit (\d+)")]
async fn when_list_endpoints_paginated(world: &mut AppWorld, offset: u64, limit: u64) {
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let endpoints = world.endpoints.take().unwrap_or_default();
    let log = OperationLog::new();
    let read_store = Arc::new(FakeEndpointReadStore::new(log, endpoints));
    let handler = ListEndpointsByAppHandler::new(read_store);

    world.list_endpoints_result = Some(
        handler
            .handle(ListEndpointsByApp {
                app_id,
                org_id,
                offset,
                limit,
            })
            .await,
    );
}

#[then("the result should be an empty endpoint list")]
async fn then_empty_ep_list(world: &mut AppWorld) {
    let result = world.list_endpoints_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.items.is_empty());
    assert_eq!(result.total, 0);
}

#[then(regex = r"the result should contain (\d+) endpoints")]
async fn then_contains_n_endpoints(world: &mut AppWorld, count: usize) {
    let result = world.list_endpoints_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
}

#[then(regex = r"the result should contain (\d+) endpoint with total (\d+)")]
async fn then_contains_n_ep_with_total(world: &mut AppWorld, count: usize, total: u64) {
    let result = world.list_endpoints_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
    assert_eq!(result.total, total);
}

// ===== Get OIDC Config by ID steps =====

#[given("an OIDC config exists in the read store")]
async fn given_oidc_in_read_store(world: &mut AppWorld) {
    let config = pigeon_domain::test_support::any_oidc_config();
    world.existing_oidc_config = Some(config);
}

#[when("the get OIDC config by id query is executed")]
async fn when_get_oidc_by_id(world: &mut AppWorld) {
    let config = world.existing_oidc_config.as_ref().unwrap().clone();
    let config_id = config.id().clone();
    let mut mock = MockOidcConfigReadStore::new();
    mock.expect_find_by_id()
        .returning(move |_| Ok(Some(config.clone())));
    let handler = GetOidcConfigByIdHandler::new(Arc::new(mock));

    world.get_oidc_config_query_result = Some(
        handler.handle(GetOidcConfigById { id: config_id }).await,
    );
}

#[when("the get OIDC config by id query is executed for a non-existent id")]
async fn when_get_oidc_by_id_nonexistent(world: &mut AppWorld) {
    let mut mock = MockOidcConfigReadStore::new();
    mock.expect_find_by_id().returning(|_| Ok(None));
    let handler = GetOidcConfigByIdHandler::new(Arc::new(mock));

    world.get_oidc_config_query_result = Some(
        handler.handle(GetOidcConfigById { id: OidcConfigId::new() }).await,
    );
}

#[then("the OIDC config should be returned")]
async fn then_oidc_returned(world: &mut AppWorld) {
    let result = world.get_oidc_config_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_some());
}

#[then("no OIDC config should be returned")]
async fn then_no_oidc_returned(world: &mut AppWorld) {
    let result = world.get_oidc_config_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_none());
}

// ===== List OIDC Configs by Org steps =====

#[given("an organization with no OIDC configs")]
async fn given_org_no_oidc_configs(world: &mut AppWorld) {
    world.org_id = Some(OrganizationId::new());
}

#[given(regex = r"an organization with (\d+) OIDC configs")]
async fn given_org_with_n_oidc_configs(world: &mut AppWorld, count: usize) {
    let org_id = OrganizationId::new();
    let mut configs = Vec::new();
    for i in 0..count {
        let config = OidcConfig::new(
            org_id.clone(),
            format!("https://auth{i}.example.com"),
            format!("audience-{i}"),
            format!("https://auth{i}.example.com/.well-known/jwks.json"),
        )
        .unwrap();
        configs.push(config);
    }
    world.org_id = Some(org_id);
    // Store configs in oidc_data for the mock to use
    let oidc_data = SharedOidcConfigData::default();
    for c in &configs {
        oidc_data.oidc_configs.lock().unwrap().push(c.clone());
    }
    world.oidc_data = Some(oidc_data);
}

#[when("the list OIDC configs query is executed")]
async fn when_list_oidc_configs(world: &mut AppWorld) {
    let org_id = world.org_id.as_ref().unwrap().clone();
    let configs: Vec<OidcConfig> = world
        .oidc_data
        .as_ref()
        .map(|d| d.oidc_configs.lock().unwrap().clone())
        .unwrap_or_default();
    let total = configs.len() as u64;
    let items = configs;

    let mut mock = MockOidcConfigReadStore::new();
    let items_clone = items.clone();
    mock.expect_list_by_org()
        .returning(move |_, _, _| Ok(items_clone.clone()));
    mock.expect_count_by_org()
        .returning(move |_| Ok(total));
    let handler = ListOidcConfigsByOrgHandler::new(Arc::new(mock));

    world.list_oidc_configs_result = Some(
        handler
            .handle(ListOidcConfigsByOrg {
                org_id,
                offset: 0,
                limit: 100,
            })
            .await,
    );
}

#[when(regex = r"the list OIDC configs query is executed with offset (\d+) and limit (\d+)")]
async fn when_list_oidc_configs_paginated(world: &mut AppWorld, offset: u64, limit: u64) {
    let org_id = world.org_id.as_ref().unwrap().clone();
    let all_configs: Vec<OidcConfig> = world
        .oidc_data
        .as_ref()
        .map(|d| d.oidc_configs.lock().unwrap().clone())
        .unwrap_or_default();
    let total = all_configs.len() as u64;
    let items: Vec<OidcConfig> = all_configs
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    let mut mock = MockOidcConfigReadStore::new();
    let items_clone = items.clone();
    mock.expect_list_by_org()
        .returning(move |_, _, _| Ok(items_clone.clone()));
    mock.expect_count_by_org()
        .returning(move |_| Ok(total));
    let handler = ListOidcConfigsByOrgHandler::new(Arc::new(mock));

    world.list_oidc_configs_result = Some(
        handler
            .handle(ListOidcConfigsByOrg {
                org_id,
                offset,
                limit,
            })
            .await,
    );
}

#[then("the result should be an empty OIDC config list")]
async fn then_empty_oidc_list(world: &mut AppWorld) {
    let result = world.list_oidc_configs_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.items.is_empty());
    assert_eq!(result.total, 0);
}

#[then(regex = r"the result should contain (\d+) OIDC configs")]
async fn then_contains_n_oidc_configs(world: &mut AppWorld, count: usize) {
    let result = world.list_oidc_configs_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
}

#[then(regex = r"the result should contain (\d+) OIDC config with total (\d+)")]
async fn then_contains_n_oidc_with_total(world: &mut AppWorld, count: usize, total: u64) {
    let result = world.list_oidc_configs_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
    assert_eq!(result.total, total);
}

// ===== Get Message by ID steps =====

#[given("a message exists in the read store")]
async fn given_msg_in_read_store(world: &mut AppWorld) {
    let msg = pigeon_domain::test_support::any_message();
    world.msg_data = Some(SharedMessageData::default());
    world.msg_data.as_ref().unwrap().messages.lock().unwrap().push(msg);
    world.org_id = Some(OrganizationId::new());
}

#[when("the get message by id query is executed")]
async fn when_get_msg_by_id(world: &mut AppWorld) {
    let messages = world.msg_data.as_ref().unwrap().messages.lock().unwrap().clone();
    let msg = messages.first().unwrap().clone();
    let msg_id = msg.id().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();

    let mut mock = MockMessageReadStore::new();
    mock.expect_find_by_id()
        .returning(move |_, _| Ok(Some(MessageWithStatus {
            message: msg.clone(),
            attempts_created: 1,
            succeeded: 0,
            failed: 0,
            dead_lettered: 0,
        })));
    let handler = GetMessageByIdHandler::new(Arc::new(mock));

    world.get_message_query_result = Some(
        handler
            .handle(GetMessageById { id: msg_id, org_id })
            .await,
    );
}

#[when("the get message by id query is executed for a non-existent id")]
async fn when_get_msg_by_id_nonexistent(world: &mut AppWorld) {
    let mut mock = MockMessageReadStore::new();
    mock.expect_find_by_id().returning(|_, _| Ok(None));
    let handler = GetMessageByIdHandler::new(Arc::new(mock));

    world.get_message_query_result = Some(
        handler
            .handle(GetMessageById {
                id: MessageId::new(),
                org_id: OrganizationId::new(),
            })
            .await,
    );
}

#[then("the message should be returned with status counts")]
async fn then_msg_returned_with_status(world: &mut AppWorld) {
    let result = world.get_message_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_some());
}

#[then("no message should be returned")]
async fn then_no_msg_returned(world: &mut AppWorld) {
    let result = world.get_message_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_none());
}

// ===== List Messages by App steps =====

#[given("an application with no messages")]
async fn given_app_no_messages(world: &mut AppWorld) {
    world.app_id = Some(ApplicationId::new());
    world.org_id = Some(OrganizationId::new());
}

#[given(regex = r"an application with (\d+) messages")]
async fn given_app_with_n_messages(world: &mut AppWorld, count: usize) {
    let app_id = ApplicationId::new();
    let org_id = OrganizationId::new();
    let msg_data = SharedMessageData::default();
    for _ in 0..count {
        let msg = pigeon_domain::test_support::any_message();
        msg_data.messages.lock().unwrap().push(msg);
    }
    world.app_id = Some(app_id);
    world.org_id = Some(org_id);
    world.msg_data = Some(msg_data);
}

#[when("the list messages query is executed")]
async fn when_list_messages(world: &mut AppWorld) {
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let messages: Vec<MessageWithStatus> = world
        .msg_data
        .as_ref()
        .map(|d| {
            d.messages
                .lock()
                .unwrap()
                .iter()
                .map(|m| MessageWithStatus {
                    message: m.clone(),
                    attempts_created: 0,
                    succeeded: 0,
                    failed: 0,
                    dead_lettered: 0,
                })
                .collect()
        })
        .unwrap_or_default();
    let total = messages.len() as u64;

    let mut mock = MockMessageReadStore::new();
    let items_clone = messages.clone();
    mock.expect_list_by_app()
        .returning(move |_, _, _, _, _, _| Ok(items_clone.clone()));
    mock.expect_count_by_app()
        .returning(move |_, _, _, _| Ok(total));
    let handler = ListMessagesByAppHandler::new(Arc::new(mock));

    world.list_messages_result = Some(
        handler
            .handle(ListMessagesByApp {
                app_id,
                org_id,
                event_type_id: None,
                status: None,
                offset: 0,
                limit: 100,
            })
            .await,
    );
}

#[when(regex = r"the list messages query is executed with offset (\d+) and limit (\d+)")]
async fn when_list_messages_paginated(world: &mut AppWorld, offset: u64, limit: u64) {
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let all_messages: Vec<MessageWithStatus> = world
        .msg_data
        .as_ref()
        .map(|d| {
            d.messages
                .lock()
                .unwrap()
                .iter()
                .map(|m| MessageWithStatus {
                    message: m.clone(),
                    attempts_created: 0,
                    succeeded: 0,
                    failed: 0,
                    dead_lettered: 0,
                })
                .collect()
        })
        .unwrap_or_default();
    let total = all_messages.len() as u64;
    let items: Vec<MessageWithStatus> = all_messages
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    let mut mock = MockMessageReadStore::new();
    let items_clone = items.clone();
    mock.expect_list_by_app()
        .returning(move |_, _, _, _, _, _| Ok(items_clone.clone()));
    mock.expect_count_by_app()
        .returning(move |_, _, _, _| Ok(total));
    let handler = ListMessagesByAppHandler::new(Arc::new(mock));

    world.list_messages_result = Some(
        handler
            .handle(ListMessagesByApp {
                app_id,
                org_id,
                event_type_id: None,
                status: None,
                offset,
                limit,
            })
            .await,
    );
}

#[then("the result should be an empty message list")]
async fn then_empty_msg_list(world: &mut AppWorld) {
    let result = world.list_messages_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.items.is_empty());
    assert_eq!(result.total, 0);
}

#[then(regex = r"the result should contain (\d+) messages")]
async fn then_contains_n_messages(world: &mut AppWorld, count: usize) {
    let result = world.list_messages_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
}

#[then(regex = r"the result should contain (\d+) message with total (\d+)")]
async fn then_contains_n_msg_with_total(world: &mut AppWorld, count: usize, total: u64) {
    let result = world.list_messages_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
    assert_eq!(result.total, total);
}

// ===== Get Dead Letter by ID steps =====

#[given("a dead letter exists in the read store")]
async fn given_dl_in_read_store(world: &mut AppWorld) {
    let dl = pigeon_domain::test_support::any_dead_letter();
    world.existing_dead_letter = Some(dl);
    world.org_id = Some(OrganizationId::new());
}

#[when("the get dead letter by id query is executed")]
async fn when_get_dl_by_id(world: &mut AppWorld) {
    let dl = world.existing_dead_letter.as_ref().unwrap().clone();
    let dl_id = dl.id().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();

    let mut mock = MockDeadLetterReadStore::new();
    mock.expect_find_by_id()
        .returning(move |_, _| Ok(Some(dl.clone())));
    let handler = GetDeadLetterByIdHandler::new(Arc::new(mock));

    world.get_dead_letter_query_result = Some(
        handler
            .handle(GetDeadLetterById { id: dl_id, org_id })
            .await,
    );
}

#[when("the get dead letter by id query is executed for a non-existent id")]
async fn when_get_dl_by_id_nonexistent(world: &mut AppWorld) {
    let mut mock = MockDeadLetterReadStore::new();
    mock.expect_find_by_id().returning(|_, _| Ok(None));
    let handler = GetDeadLetterByIdHandler::new(Arc::new(mock));

    world.get_dead_letter_query_result = Some(
        handler
            .handle(GetDeadLetterById {
                id: DeadLetterId::new(),
                org_id: OrganizationId::new(),
            })
            .await,
    );
}

#[then("the dead letter should be returned")]
async fn then_dl_returned(world: &mut AppWorld) {
    let result = world.get_dead_letter_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_some());
}

#[then("no dead letter should be returned")]
async fn then_no_dl_returned(world: &mut AppWorld) {
    let result = world.get_dead_letter_query_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_none());
}

// ===== List Dead Letters by App steps =====

#[given("an application with no dead letters")]
async fn given_app_no_dead_letters(world: &mut AppWorld) {
    world.app_id = Some(ApplicationId::new());
    world.org_id = Some(OrganizationId::new());
}

#[given(regex = r"an application with (\d+) dead letters")]
async fn given_app_with_n_dead_letters(world: &mut AppWorld, count: usize) {
    let app_id = ApplicationId::new();
    let org_id = OrganizationId::new();
    let dl_data = SharedDeadLetterData::default();
    for _ in 0..count {
        let dl = pigeon_domain::test_support::any_dead_letter();
        dl_data.dead_letters.lock().unwrap().push(dl);
    }
    world.app_id = Some(app_id);
    world.org_id = Some(org_id);
    world.dl_data = Some(dl_data);
}

#[when("the list dead letters query is executed")]
async fn when_list_dead_letters(world: &mut AppWorld) {
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let dead_letters: Vec<DeadLetter> = world
        .dl_data
        .as_ref()
        .map(|d| d.dead_letters.lock().unwrap().clone())
        .unwrap_or_default();
    let total = dead_letters.len() as u64;

    let mut mock = MockDeadLetterReadStore::new();
    let items_clone = dead_letters.clone();
    mock.expect_list_by_app()
        .returning(move |_, _, _, _, _, _| Ok(items_clone.clone()));
    mock.expect_count_by_app()
        .returning(move |_, _, _, _| Ok(total));
    let handler = ListDeadLettersByAppHandler::new(Arc::new(mock));

    world.list_dead_letters_result = Some(
        handler
            .handle(ListDeadLettersByApp {
                app_id,
                org_id,
                endpoint_id: None,
                replayed: None,
                offset: 0,
                limit: 100,
            })
            .await,
    );
}

#[when(regex = r"the list dead letters query is executed with offset (\d+) and limit (\d+)")]
async fn when_list_dead_letters_paginated(world: &mut AppWorld, offset: u64, limit: u64) {
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();
    let all_dls: Vec<DeadLetter> = world
        .dl_data
        .as_ref()
        .map(|d| d.dead_letters.lock().unwrap().clone())
        .unwrap_or_default();
    let total = all_dls.len() as u64;
    let items: Vec<DeadLetter> = all_dls
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    let mut mock = MockDeadLetterReadStore::new();
    let items_clone = items.clone();
    mock.expect_list_by_app()
        .returning(move |_, _, _, _, _, _| Ok(items_clone.clone()));
    mock.expect_count_by_app()
        .returning(move |_, _, _, _| Ok(total));
    let handler = ListDeadLettersByAppHandler::new(Arc::new(mock));

    world.list_dead_letters_result = Some(
        handler
            .handle(ListDeadLettersByApp {
                app_id,
                org_id,
                endpoint_id: None,
                replayed: None,
                offset,
                limit,
            })
            .await,
    );
}

#[then("the result should be an empty dead letter list")]
async fn then_empty_dl_list(world: &mut AppWorld) {
    let result = world.list_dead_letters_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.items.is_empty());
    assert_eq!(result.total, 0);
}

#[then(regex = r"the result should contain (\d+) dead letters")]
async fn then_contains_n_dead_letters(world: &mut AppWorld, count: usize) {
    let result = world.list_dead_letters_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
}

#[then(regex = r"the result should contain (\d+) dead letter with total (\d+)")]
async fn then_contains_n_dl_with_total(world: &mut AppWorld, count: usize, total: u64) {
    let result = world.list_dead_letters_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
    assert_eq!(result.total, total);
}

// ===== List Attempts by Message steps =====

#[given("a message with no delivery attempts")]
async fn given_msg_no_attempts(world: &mut AppWorld) {
    world.org_id = Some(OrganizationId::new());
    let msg = pigeon_domain::test_support::any_message();
    world.msg_data = Some(SharedMessageData::default());
    world.msg_data.as_ref().unwrap().messages.lock().unwrap().push(msg);
}

#[given(regex = r"a message with (\d+) delivery attempts")]
async fn given_msg_with_n_attempts(world: &mut AppWorld, count: usize) {
    world.org_id = Some(OrganizationId::new());
    let msg = pigeon_domain::test_support::any_message();
    let mut attempts = Vec::new();
    for _ in 0..count {
        attempts.push(pigeon_domain::test_support::any_attempt());
    }
    world.msg_data = Some(SharedMessageData::default());
    world.msg_data.as_ref().unwrap().messages.lock().unwrap().push(msg);
    world.existing_attempts = Some(attempts);
}

#[when("the list attempts by message query is executed")]
async fn when_list_attempts(world: &mut AppWorld) {
    let org_id = world.org_id.as_ref().unwrap().clone();
    let messages = world.msg_data.as_ref().unwrap().messages.lock().unwrap().clone();
    let msg_id = messages.first().unwrap().id().clone();
    let attempts = world.existing_attempts.take().unwrap_or_default();

    let mut mock = MockAttemptReadStoreImport::new();
    mock.expect_list_by_message()
        .returning(move |_, _| Ok(attempts.clone()));
    let handler = ListAttemptsByMessageHandler::new(Arc::new(mock));

    world.list_attempts_result = Some(
        handler
            .handle(ListAttemptsByMessage {
                message_id: msg_id,
                org_id,
            })
            .await,
    );
}

#[then("the result should be an empty attempt list")]
async fn then_empty_attempt_list(world: &mut AppWorld) {
    let result = world.list_attempts_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.is_empty());
}

#[then(regex = r"the result should contain (\d+) attempts")]
async fn then_contains_n_attempts(world: &mut AppWorld, count: usize) {
    let result = world.list_attempts_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.len(), count);
}

// ===== List Audit Log steps =====

#[given("an organization with no audit entries")]
async fn given_org_no_audit_entries(world: &mut AppWorld) {
    world.org_id = Some(OrganizationId::new());
}

#[given(regex = r"an organization with (\d+) audit entries")]
async fn given_org_with_n_audit_entries(world: &mut AppWorld, count: usize) {
    let org_id = OrganizationId::new();
    let mut entries = Vec::new();
    for _ in 0..count {
        entries.push(AuditLogEntry {
            id: uuid::Uuid::new_v4(),
            command_name: "TestCommand".to_string(),
            actor: "test-user".to_string(),
            org_id: org_id.clone(),
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
        });
    }
    world.org_id = Some(org_id);
    // Store entries temporarily via existing_attempts field repurposed — we'll use a closure
    // Actually, store in a new approach: build mock in the when step. Store count.
    // We need to carry the entries forward. Use a simple approach: re-create in when step.
    // Let's just store the count in the org name trick... no, let's properly handle it.
    // We'll reconstruct in the when step based on the org_id.
    // Actually we can use the audit entries directly in a helper field. But we don't have one.
    // Simplest: just store the entries in a local variable captured by the mock.
    // We need to pass data from given to when. Let's use the log field as a side channel.
    let log = OperationLog::new();
    for _ in 0..count {
        log.record("audit_entry_placeholder");
    }
    world.log = Some(log);
}

#[when("the list audit log query is executed")]
async fn when_list_audit_log(world: &mut AppWorld) {
    let org_id = world.org_id.as_ref().unwrap().clone();
    let count = world
        .log
        .as_ref()
        .map(|l| l.entries().iter().filter(|e| e == &"audit_entry_placeholder").count())
        .unwrap_or(0);

    let org_id_for_entries = org_id.clone();
    let entries: Vec<AuditLogEntry> = (0..count)
        .map(|_| AuditLogEntry {
            id: uuid::Uuid::new_v4(),
            command_name: "TestCommand".to_string(),
            actor: "test-user".to_string(),
            org_id: org_id_for_entries.clone(),
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
        })
        .collect();
    let total = entries.len() as u64;

    let mut mock = MockAuditReadStore::new();
    let items_clone = entries.clone();
    mock.expect_list_by_org()
        .returning(move |_, _, _, _, _| Ok(items_clone.clone()));
    mock.expect_count_by_org()
        .returning(move |_, _, _| Ok(total));
    let handler = ListAuditLogHandler::new(Arc::new(mock));

    world.list_audit_log_result = Some(
        handler
            .handle(ListAuditLog {
                org_id,
                command_filter: None,
                success_filter: None,
                offset: 0,
                limit: 100,
            })
            .await,
    );
}

#[when(regex = r"the list audit log query is executed with offset (\d+) and limit (\d+)")]
async fn when_list_audit_log_paginated(world: &mut AppWorld, offset: u64, limit: u64) {
    let org_id = world.org_id.as_ref().unwrap().clone();
    let count = world
        .log
        .as_ref()
        .map(|l| l.entries().iter().filter(|e| e == &"audit_entry_placeholder").count())
        .unwrap_or(0);

    let org_id_for_entries = org_id.clone();
    let all_entries: Vec<AuditLogEntry> = (0..count)
        .map(|_| AuditLogEntry {
            id: uuid::Uuid::new_v4(),
            command_name: "TestCommand".to_string(),
            actor: "test-user".to_string(),
            org_id: org_id_for_entries.clone(),
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
        })
        .collect();
    let total = all_entries.len() as u64;
    let items: Vec<AuditLogEntry> = all_entries
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    let mut mock = MockAuditReadStore::new();
    let items_clone = items.clone();
    mock.expect_list_by_org()
        .returning(move |_, _, _, _, _| Ok(items_clone.clone()));
    mock.expect_count_by_org()
        .returning(move |_, _, _| Ok(total));
    let handler = ListAuditLogHandler::new(Arc::new(mock));

    world.list_audit_log_result = Some(
        handler
            .handle(ListAuditLog {
                org_id,
                command_filter: None,
                success_filter: None,
                offset,
                limit,
            })
            .await,
    );
}

#[then("the result should be an empty audit log list")]
async fn then_empty_audit_list(world: &mut AppWorld) {
    let result = world.list_audit_log_result.as_ref().unwrap().as_ref().unwrap();
    assert!(result.items.is_empty());
    assert_eq!(result.total, 0);
}

#[then(regex = r"the result should contain (\d+) audit entries")]
async fn then_contains_n_audit_entries(world: &mut AppWorld, count: usize) {
    let result = world.list_audit_log_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
}

#[then(regex = r"the result should contain (\d+) audit entry with total (\d+)")]
async fn then_contains_n_audit_with_total(world: &mut AppWorld, count: usize, total: u64) {
    let result = world.list_audit_log_result.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(result.items.len(), count);
    assert_eq!(result.total, total);
}

// ===== Get App Stats steps =====

#[given("an application with delivery statistics")]
async fn given_app_with_stats(world: &mut AppWorld) {
    world.app_id = Some(ApplicationId::new());
    world.org_id = Some(OrganizationId::new());
}

#[when("the get app stats query is executed")]
async fn when_get_app_stats(world: &mut AppWorld) {
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();

    let stats = AppStats {
        total_messages: 10,
        total_attempts: 20,
        total_pending: 2,
        total_succeeded: 15,
        total_failed: 2,
        total_dead_lettered: 1,
        success_rate: 75.0,
        time_series: vec![],
    };

    let mut mock = MockStatsReadStore::new();
    let stats_clone = stats.clone();
    mock.expect_get_app_stats()
        .returning(move |_, _, _, _| Ok(stats_clone.clone()));
    let handler = GetAppStatsHandler::new(Arc::new(mock));

    world.get_app_stats_result = Some(
        handler
            .handle(GetAppStats {
                app_id,
                org_id,
                since: chrono::Utc::now() - chrono::Duration::hours(24),
                bucket_interval_hours: 1,
            })
            .await,
    );
}

#[then("the application statistics should be returned")]
async fn then_app_stats_returned(world: &mut AppWorld) {
    let result = world.get_app_stats_result.as_ref().unwrap();
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let stats = result.as_ref().unwrap();
    assert_eq!(stats.total_messages, 10);
}

// ===== Get Endpoint Stats steps =====

#[given("an endpoint with delivery statistics")]
async fn given_ep_with_stats(world: &mut AppWorld) {
    let ep = pigeon_domain::test_support::any_endpoint();
    world.existing_endpoint = Some(ep);
    world.org_id = Some(OrganizationId::new());
}

#[when("the get endpoint stats query is executed")]
async fn when_get_endpoint_stats(world: &mut AppWorld) {
    let ep = world.existing_endpoint.as_ref().unwrap();
    let org_id = world.org_id.as_ref().unwrap().clone();

    let stats = EndpointStats {
        total_attempts: 10,
        total_pending: 1,
        total_succeeded: 8,
        total_failed: 1,
        total_dead_lettered: 0,
        success_rate: 80.0,
        consecutive_failures: 0,
        last_delivery_at: None,
        last_status: None,
        time_series: vec![],
    };

    let mut mock = MockEndpointStatsReadStore::new();
    let stats_clone = stats.clone();
    mock.expect_get_stats()
        .returning(move |_, _, _, _| Ok(stats_clone.clone()));
    let handler = GetEndpointStatsHandler::new(Arc::new(mock));

    world.get_endpoint_stats_result = Some(
        handler
            .handle(GetEndpointStats {
                endpoint_id: ep.id().clone(),
                org_id,
                since: chrono::Utc::now() - chrono::Duration::hours(24),
                bucket_interval_hours: 1,
            })
            .await,
    );
}

#[then("the endpoint statistics should be returned")]
async fn then_endpoint_stats_returned(world: &mut AppWorld) {
    let result = world.get_endpoint_stats_result.as_ref().unwrap();
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let stats = result.as_ref().unwrap();
    assert_eq!(stats.total_attempts, 10);
}

// ===== Get Event Type Stats steps =====

#[given("an event type with delivery statistics")]
async fn given_et_with_stats(world: &mut AppWorld) {
    let et = pigeon_domain::test_support::any_event_type();
    world.existing_event_type = Some(et);
    world.app_id = Some(ApplicationId::new());
    world.org_id = Some(OrganizationId::new());
}

#[when("the get event type stats query is executed")]
async fn when_get_event_type_stats(world: &mut AppWorld) {
    let et = world.existing_event_type.as_ref().unwrap();
    let app_id = world.app_id.as_ref().unwrap().clone();
    let org_id = world.org_id.as_ref().unwrap().clone();

    let stats = EventTypeStats {
        total_messages: 5,
        total_attempts: 10,
        total_pending: 0,
        total_succeeded: 9,
        total_failed: 1,
        total_dead_lettered: 0,
        success_rate: 90.0,
        subscribed_endpoints: 3,
        time_series: vec![],
        recent_messages: vec![],
    };

    let mut mock = MockEventTypeStatsReadStore::new();
    let stats_clone = stats.clone();
    mock.expect_get_stats()
        .returning(move |_, _, _, _, _| Ok(stats_clone.clone()));
    let handler = GetEventTypeStatsHandler::new(Arc::new(mock));

    world.get_event_type_stats_result = Some(
        handler
            .handle(GetEventTypeStats {
                app_id,
                event_type_id: et.id().clone(),
                org_id,
                since: chrono::Utc::now() - chrono::Duration::hours(24),
                bucket_interval_hours: 1,
            })
            .await,
    );
}

#[then("the event type statistics should be returned")]
async fn then_event_type_stats_returned(world: &mut AppWorld) {
    let result = world.get_event_type_stats_result.as_ref().unwrap();
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let stats = result.as_ref().unwrap();
    assert_eq!(stats.total_messages, 5);
}

#[tokio::main]
async fn main() {
    AppWorld::cucumber()
        .with_default_cli()
        .run("tests/features")
        .await;
}
