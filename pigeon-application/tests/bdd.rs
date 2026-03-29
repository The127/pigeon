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
use pigeon_application::commands::retrigger_message::{
    RetriggerMessage, RetriggerMessageHandler, RetriggerMessageResult,
};
use pigeon_application::commands::send_message::{
    SendMessage, SendMessageHandler, SendMessageResult,
};
use pigeon_application::ports::stores::MockMessageReadStore;
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
use pigeon_application::ports::unit_of_work::UnitOfWorkFactory;
use pigeon_application::test_support::fakes::{
    FakeApplicationReadStore, FakeEndpointReadStore, FakeUnitOfWorkFactory, OperationLog,
    SharedApplicationData, SharedEndpointData, SharedEventTypeData, SharedMessageData,
    SharedOidcConfigData, SharedOrganizationData,
};
use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::endpoint::{Endpoint, EndpointId};
use pigeon_domain::event_type::{EventType, EventTypeId};
use pigeon_domain::message::Message;
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
    let handler = CreateApplicationHandler::new(factory);
    let command = world.command.take().unwrap();
    world.result = Some(handler.handle(command).await);
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
    let handler = UpdateApplicationHandler::new(factory);

    world.update_result = Some(
        handler
            .handle(UpdateApplication {
                org_id,
                id: app.id().clone(),
                name,
                version: app.version(),
            })
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
    let handler = UpdateApplicationHandler::new(factory);

    world.update_result = Some(
        handler
            .handle(UpdateApplication {
                org_id,
                id: app.id().clone(),
                name: "new-name".into(),
                version: Version::new(999),
            })
            .await,
    );
    world.log = Some(log);
}

#[when("the update application command is executed for a non-existent application")]
async fn when_update_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = UpdateApplicationHandler::new(factory);

    world.update_result = Some(
        handler
            .handle(UpdateApplication {
                org_id: OrganizationId::new(),
                id: ApplicationId::new(),
                name: "new-name".into(),
                version: Version::new(0),
            })
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
    let handler = DeleteApplicationHandler::new(factory);

    world.delete_result = Some(
        handler
            .handle(DeleteApplication {
                org_id,
                id: app.id().clone(),
            })
            .await,
    );
    world.log = Some(log);
}

#[when("the delete application command is executed for a non-existent application")]
async fn when_delete_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteApplicationHandler::new(factory);

    world.delete_result = Some(
        handler
            .handle(DeleteApplication {
                org_id: OrganizationId::new(),
                id: ApplicationId::new(),
            })
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
    let org_id = world.org_id.clone().unwrap_or_else(OrganizationId::new);
    let read_store = Arc::new(FakeApplicationReadStore::new(log.clone(), data));
    let handler = ListApplicationsHandler::new(read_store);

    world.list_result = Some(
        handler
            .handle(ListApplications { org_id, offset, limit })
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
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
    let handler = CreateEventTypeHandler::new(factory);
    let command = world.create_event_type_command.take().unwrap();
    world.create_event_type_result = Some(handler.handle(command).await);
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
    let handler = UpdateEventTypeHandler::new(factory);

    world.update_event_type_result = Some(
        handler
            .handle(UpdateEventType {
                org_id: OrganizationId::new(),
                id: et.id().clone(),
                name,
                schema: None,
                version: et.version(),
            })
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
    let handler = UpdateEventTypeHandler::new(factory);

    world.update_event_type_result = Some(
        handler
            .handle(UpdateEventType {
                org_id: OrganizationId::new(),
                id: et.id().clone(),
                name: "new.event".into(),
                schema: None,
                version: Version::new(999),
            })
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
    let handler = DeleteEventTypeHandler::new(factory);

    world.delete_event_type_result = Some(
        handler
            .handle(DeleteEventType {
                org_id: OrganizationId::new(),
                id: et.id().clone(),
            })
            .await,
    );
    world.log = Some(log);
}

#[when("the delete event type command is executed for a non-existent event type")]
async fn when_delete_et_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteEventTypeHandler::new(factory);

    world.delete_event_type_result = Some(
        handler
            .handle(DeleteEventType {
                org_id: OrganizationId::new(),
                id: EventTypeId::new(),
            })
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
    world.create_endpoint_command = Some(CreateEndpoint {
        org_id: OrganizationId::new(),
        app_id: ApplicationId::new(),
        url,
        signing_secret: "whsec_secret123".into(),
        event_type_ids: vec![EventTypeId::new()],
    });
    world.log = Some(OperationLog::new());
}

#[given(regex = r#"a request to create an endpoint with url "([^"]*)" and signing secret "([^"]*)""#)]
async fn given_create_ep_with_secret(world: &mut AppWorld, url: String, signing_secret: String) {
    world.create_endpoint_command = Some(CreateEndpoint {
        org_id: OrganizationId::new(),
        app_id: ApplicationId::new(),
        url,
        signing_secret,
        event_type_ids: vec![EventTypeId::new()],
    });
    world.log = Some(OperationLog::new());
}

#[when("the create endpoint command is executed")]
async fn when_create_ep_executed(world: &mut AppWorld) {
    let log = world.log.as_ref().unwrap().clone();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log));
    let handler = CreateEndpointHandler::new(factory);
    let command = world.create_endpoint_command.take().unwrap();
    world.create_endpoint_result = Some(handler.handle(command).await);
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
        url,
        "whsec_secret123".into(),
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
    let handler = UpdateEndpointHandler::new(factory);

    world.update_endpoint_result = Some(
        handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id: ep.id().clone(),
                url,
                signing_secret: "whsec_secret123".into(),
                event_type_ids: vec![],
                version: ep.version(),
            })
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
    let handler = UpdateEndpointHandler::new(factory);

    world.update_endpoint_result = Some(
        handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id: ep.id().clone(),
                url: "https://new.example.com/webhook".into(),
                signing_secret: "whsec_secret123".into(),
                event_type_ids: vec![],
                version: Version::new(999),
            })
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
    let handler = UpdateEndpointHandler::new(factory);

    world.update_endpoint_result = Some(
        handler
            .handle(UpdateEndpoint {
                org_id: OrganizationId::new(),
                id: EndpointId::new(),
                url: "https://example.com/webhook".into(),
                signing_secret: "whsec_secret".into(),
                event_type_ids: vec![],
                version: Version::new(0),
            })
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
        url,
        "whsec_secret123".into(),
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
    let handler = DeleteEndpointHandler::new(factory);

    world.delete_endpoint_result = Some(
        handler
            .handle(DeleteEndpoint {
                org_id: OrganizationId::new(),
                id: ep.id().clone(),
            })
            .await,
    );
    world.log = Some(log);
}

#[when("the delete endpoint command is executed for a non-existent endpoint")]
async fn when_delete_ep_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteEndpointHandler::new(factory);

    world.delete_endpoint_result = Some(
        handler
            .handle(DeleteEndpoint {
                org_id: OrganizationId::new(),
                id: EndpointId::new(),
            })
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
        "https://a.com/hook".into(),
        "whsec_a".into(),
        vec![event_type_id.clone()],
    )
    .unwrap();
    let ep2 = Endpoint::new(
        app_id.clone(),
        "https://b.com/hook".into(),
        "whsec_b".into(),
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
        factory,
        endpoint_store,
        chrono::Duration::hours(24),
    );

    world.send_message_result = Some(
        handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!({"data": true}),
                idempotency_key: None,
            })
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
        factory,
        endpoint_store,
        chrono::Duration::hours(24),
    );

    world.send_message_result = Some(
        handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!({"data": true}),
                idempotency_key: Some(key),
            })
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
        factory,
        endpoint_store,
        chrono::Duration::hours(24),
    );

    world.send_message_result = Some(
        handler
            .handle(SendMessage {
                org_id: OrganizationId::new(),
                app_id,
                event_type_id,
                payload: json!("not an object"),
                idempotency_key: None,
            })
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

    let handler = CreateOrganizationHandler::new(factory.clone());
    world.create_org_result = Some(handler.handle(command).await);
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
    let handler = UpdateOrganizationHandler::new(factory);

    world.update_org_result = Some(
        handler
            .handle(UpdateOrganization {
                id: org.id().clone(),
                name,
                version: org.version(),
            })
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
    let handler = UpdateOrganizationHandler::new(factory);

    world.update_org_result = Some(
        handler
            .handle(UpdateOrganization {
                id: org.id().clone(),
                name: "new-name".into(),
                version: Version::new(999),
            })
            .await,
    );
    world.log = Some(log);
}

#[when("the update organization command is executed for a non-existent organization")]
async fn when_update_org_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = UpdateOrganizationHandler::new(factory);

    world.update_org_result = Some(
        handler
            .handle(UpdateOrganization {
                id: OrganizationId::new(),
                name: "new-name".into(),
                version: Version::new(0),
            })
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
    let handler = DeleteOrganizationHandler::new(factory);

    world.delete_org_result = Some(
        handler
            .handle(DeleteOrganization {
                id: org.id().clone(),
            })
            .await,
    );
    world.log = Some(log);
}

#[when("the delete organization command is executed for a non-existent organization")]
async fn when_delete_org_nonexistent(world: &mut AppWorld) {
    let log = OperationLog::new();
    let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
    let handler = DeleteOrganizationHandler::new(factory);

    world.delete_org_result = Some(
        handler
            .handle(DeleteOrganization {
                id: OrganizationId::new(),
            })
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
    let handler = CreateOidcConfigHandler::new(factory);

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
            })
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
    let handler = DeleteOidcConfigHandler::new(factory);

    world.delete_oidc_config_result = Some(
        handler
            .handle(DeleteOidcConfig {
                id: config.id().clone(),
            })
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
    let handler = DeleteOidcConfigHandler::new(factory);

    world.delete_oidc_config_result = Some(
        handler
            .handle(DeleteOidcConfig {
                id: OidcConfigId::new(),
            })
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
        "https://ep.com/hook".into(),
        "whsec_test".into(),
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
        "https://ep.com/hook".into(),
        "whsec_test".into(),
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
        factory,
        Arc::new(mock_msg_store),
        endpoint_store,
        Arc::new(mock_att_store),
    );

    world.retrigger_result = Some(
        handler
            .handle(RetriggerMessage { message_id, org_id })
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
        factory,
        Arc::new(mock_msg_store),
        endpoint_store,
        Arc::new(mock_att_store),
    );

    world.retrigger_result = Some(
        handler
            .handle(RetriggerMessage {
                message_id: pigeon_domain::message::MessageId::new(),
                org_id: OrganizationId::new(),
            })
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

#[tokio::main]
async fn main() {
    AppWorld::cucumber()
        .with_default_cli()
        .run("tests/features")
        .await;
}
