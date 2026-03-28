use pigeon_application::error::ApplicationError;
use pigeon_application::ports::delivery::DeliveryQueue;
use pigeon_application::ports::stores::{ApplicationReadStore, EndpointReadStore, EventTypeReadStore, OidcConfigReadStore, OrganizationReadStore};
use pigeon_application::ports::unit_of_work::UnitOfWorkFactory;
use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::attempt::Attempt;
use pigeon_domain::endpoint::Endpoint;
use pigeon_domain::event_type::EventType;
use pigeon_domain::oidc_config::OidcConfig;
use pigeon_domain::organization::{Organization, OrganizationId};
use pigeon_infrastructure::persistence::{
    PgApplicationReadStore, PgDeliveryQueue, PgEndpointReadStore, PgEventTypeReadStore,
    PgOidcConfigReadStore, PgOrganizationReadStore, PgUnitOfWorkFactory,
};
use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

async fn setup() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
    let container = Postgres::default().start().await.unwrap();
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432).await.unwrap()
    );
    let pool = PgPool::connect(&connection_string).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    (pool, container)
}

/// Helper: create an organization in the DB and return its ID (applications FK to organizations).
async fn insert_org(factory: &PgUnitOfWorkFactory) -> OrganizationId {
    let org = Organization::new(
        format!("org-{}", uuid::Uuid::new_v4()),
        format!("org-{}", uuid::Uuid::new_v4().simple()),
    )
    .unwrap();
    let id = org.id().clone();
    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().insert(&org).await.unwrap();
    uow.commit().await.unwrap();
    id
}

fn any_application_for_org(org_id: &OrganizationId, name: &str) -> Application {
    Application::new(org_id.clone(), name.to_string(), format!("uid_{}", uuid::Uuid::new_v4())).unwrap()
}

// ---------------------------------------------------------------------------
// PgApplicationStore + PgUnitOfWork tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_and_find_by_id() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "test-app");

    // Insert via UoW
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    // Find via new UoW
    let mut uow = factory.begin().await.unwrap();
    let found = uow.application_store().find_by_id(app.id()).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name(), "test-app");
    assert_eq!(found.uid(), app.uid());
}

#[tokio::test]
async fn find_by_id_returns_none_for_nonexistent() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);

    let mut uow = factory.begin().await.unwrap();
    let result = uow
        .application_store()
        .find_by_id(&ApplicationId::new())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn save_updates_application() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "original");

    // Insert
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    // Load, rename, save
    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .application_store()
        .find_by_id(app.id())
        .await
        .unwrap()
        .unwrap();
    loaded.rename("renamed".to_string()).unwrap();
    uow.application_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    // Verify
    let mut uow = factory.begin().await.unwrap();
    let found = uow
        .application_store()
        .find_by_id(app.id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.name(), "renamed");
}

#[tokio::test]
async fn save_with_stale_version_returns_conflict() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "app");

    // Insert
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    // Load and update to advance version
    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .application_store()
        .find_by_id(app.id())
        .await
        .unwrap()
        .unwrap();
    loaded.rename("updated".to_string()).unwrap();
    uow.application_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    // Try to save with the original (stale) version — still has version from insert
    // We need an app with the old version. The originally inserted `app` has version 0.
    // After the UPDATE above, xmin changed, so version 0 is stale.
    let mut stale = app.clone();
    stale.rename("stale-update".to_string()).unwrap();
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().save(&stale).await.unwrap();
    let result = uow.commit().await;

    assert!(matches!(result, Err(ApplicationError::Conflict)));
}

#[tokio::test]
async fn delete_removes_application() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "to-delete");

    // Insert
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    // Delete
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().delete(app.id()).await.unwrap();
    uow.commit().await.unwrap();

    // Verify gone
    let mut uow = factory.begin().await.unwrap();
    let found = uow.application_store().find_by_id(app.id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn find_by_id_overlays_pending_insert() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "pending");

    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    // Without committing, find_by_id should return the pending insert
    let found = uow.application_store().find_by_id(app.id()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), "pending");
}

#[tokio::test]
async fn find_by_id_overlays_pending_delete() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "will-delete");

    // Insert and commit
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    // In new UoW: delete then find should return None
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().delete(app.id()).await.unwrap();
    let found = uow.application_store().find_by_id(app.id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn commit_with_no_changes_succeeds() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);

    let uow = factory.begin().await.unwrap();
    uow.commit().await.unwrap();
}

#[tokio::test]
async fn rollback_discards_changes() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "rollback-me");

    // Insert via UoW, then rollback
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.rollback().await.unwrap();

    // Verify not in DB
    let mut uow = factory.begin().await.unwrap();
    let found = uow.application_store().find_by_id(app.id()).await.unwrap();
    assert!(found.is_none());
}

// ---------------------------------------------------------------------------
// PgApplicationReadStore tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn read_store_find_by_id_returns_application() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgApplicationReadStore::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "readable");

    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    let found = read_store.find_by_id(app.id()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), "readable");
}

#[tokio::test]
async fn read_store_find_by_id_returns_none() {
    let (pool, _container) = setup().await;
    let read_store = PgApplicationReadStore::new(pool);

    let result = read_store.find_by_id(&ApplicationId::new()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn read_store_list_returns_applications_ordered_by_created_at_desc() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgApplicationReadStore::new(pool);
    let org_id = insert_org(&factory).await;

    // Insert three apps with slight time separation
    let apps: Vec<Application> = (0..3)
        .map(|i| any_application_for_org(&org_id, &format!("app-{i}")))
        .collect();

    for app in &apps {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(app).await.unwrap();
        uow.commit().await.unwrap();
    }

    let listed = read_store.list_by_org(&org_id, 0, 10).await.unwrap();
    assert_eq!(listed.len(), 3);
    // Ordered by created_at DESC — last inserted should be first
    for i in 0..listed.len() - 1 {
        assert!(listed[i].created_at() >= listed[i + 1].created_at());
    }
}

#[tokio::test]
async fn read_store_list_respects_offset_and_limit() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgApplicationReadStore::new(pool);
    let org_id = insert_org(&factory).await;

    // Insert 5 apps
    for i in 0..5 {
        let app = any_application_for_org(&org_id, &format!("paged-{i}"));
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app).await.unwrap();
        uow.commit().await.unwrap();
    }

    let page = read_store.list_by_org(&org_id, 2, 2).await.unwrap();
    assert_eq!(page.len(), 2);
}

#[tokio::test]
async fn read_store_count_returns_total() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgApplicationReadStore::new(pool);
    let org_id = insert_org(&factory).await;

    for i in 0..3 {
        let app = any_application_for_org(&org_id, &format!("counted-{i}"));
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app).await.unwrap();
        uow.commit().await.unwrap();
    }

    let count = read_store.count_by_org(&org_id).await.unwrap();
    assert_eq!(count, 3);
}

// ---------------------------------------------------------------------------
// Version / xmin tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn version_changes_after_update() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgApplicationReadStore::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "versioned");

    // Insert
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    let v1 = read_store
        .find_by_id(app.id())
        .await
        .unwrap()
        .unwrap()
        .version();

    // Update
    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .application_store()
        .find_by_id(app.id())
        .await
        .unwrap()
        .unwrap();
    loaded.rename("versioned-v2".to_string()).unwrap();
    uow.application_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    let v2 = read_store
        .find_by_id(app.id())
        .await
        .unwrap()
        .unwrap()
        .version();

    assert_ne!(v1, v2);
}

#[tokio::test]
async fn concurrent_update_detects_conflict() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "concurrent");

    // Insert
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    // Read in two UoWs
    let mut uow1 = factory.begin().await.unwrap();
    let mut loaded1 = uow1
        .application_store()
        .find_by_id(app.id())
        .await
        .unwrap()
        .unwrap();

    let mut uow2 = factory.begin().await.unwrap();
    let mut loaded2 = uow2
        .application_store()
        .find_by_id(app.id())
        .await
        .unwrap()
        .unwrap();

    // Update in first UoW and commit
    loaded1.rename("first-update".to_string()).unwrap();
    uow1.application_store().save(&loaded1).await.unwrap();
    uow1.commit().await.unwrap();

    // Update in second UoW — should conflict because xmin changed
    loaded2.rename("second-update".to_string()).unwrap();
    uow2.application_store().save(&loaded2).await.unwrap();
    let result = uow2.commit().await;

    assert!(matches!(result, Err(ApplicationError::Conflict)));
}

// ---------------------------------------------------------------------------
// PgEventTypeStore + PgUnitOfWork tests
// ---------------------------------------------------------------------------

/// Helper: insert an application and return it (event_types has FK to applications).
async fn insert_application(factory: &PgUnitOfWorkFactory) -> Application {
    let org_id = insert_org(factory).await;
    let app = any_application_for_org(&org_id, "et-parent");
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();
    app
}

fn any_event_type_for(app: &Application, name: &str) -> EventType {
    EventType::new(app.id().clone(), name.to_string(), None).unwrap()
}

#[tokio::test]
async fn insert_and_find_event_type() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let app = insert_application(&factory).await;
    let et = any_event_type_for(&app, "user.created");

    let mut uow = factory.begin().await.unwrap();
    uow.event_type_store().insert(&et).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let found = uow.event_type_store().find_by_id(et.id(), app.org_id()).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name(), "user.created");
    assert_eq!(found.app_id(), app.id());
}

#[tokio::test]
async fn save_updates_event_type() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let app = insert_application(&factory).await;
    let et = any_event_type_for(&app, "original.event");

    let mut uow = factory.begin().await.unwrap();
    uow.event_type_store().insert(&et).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .event_type_store()
        .find_by_id(et.id(), app.org_id())
        .await
        .unwrap()
        .unwrap();
    loaded.update("renamed.event".to_string(), None).unwrap();
    uow.event_type_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let found = uow
        .event_type_store()
        .find_by_id(et.id(), app.org_id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.name(), "renamed.event");
}

#[tokio::test]
async fn delete_removes_event_type() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let app = insert_application(&factory).await;
    let et = any_event_type_for(&app, "to-delete");

    let mut uow = factory.begin().await.unwrap();
    uow.event_type_store().insert(&et).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    uow.event_type_store().delete(et.id()).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let found = uow.event_type_store().find_by_id(et.id(), app.org_id()).await.unwrap();
    assert!(found.is_none());
}

// ---------------------------------------------------------------------------
// PgEventTypeReadStore tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn read_store_list_event_types_by_app() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgEventTypeReadStore::new(pool);
    let app = insert_application(&factory).await;

    for i in 0..3 {
        let et = any_event_type_for(&app, &format!("event.{i}"));
        let mut uow = factory.begin().await.unwrap();
        uow.event_type_store().insert(&et).await.unwrap();
        uow.commit().await.unwrap();
    }

    let listed = read_store.list_by_app(app.id(), app.org_id(), 0, 10).await.unwrap();
    assert_eq!(listed.len(), 3);
    // Ordered by created_at DESC
    for i in 0..listed.len() - 1 {
        assert!(listed[i].created_at() >= listed[i + 1].created_at());
    }
}

#[tokio::test]
async fn read_store_count_event_types_by_app() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgEventTypeReadStore::new(pool);
    let app = insert_application(&factory).await;

    for i in 0..3 {
        let et = any_event_type_for(&app, &format!("counted.{i}"));
        let mut uow = factory.begin().await.unwrap();
        uow.event_type_store().insert(&et).await.unwrap();
        uow.commit().await.unwrap();
    }

    let count = read_store.count_by_app(app.id(), app.org_id()).await.unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn version_changes_after_event_type_update() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgEventTypeReadStore::new(pool);
    let app = insert_application(&factory).await;
    let et = any_event_type_for(&app, "versioned");

    let mut uow = factory.begin().await.unwrap();
    uow.event_type_store().insert(&et).await.unwrap();
    uow.commit().await.unwrap();

    let v1 = read_store
        .find_by_id(et.id(), app.org_id())
        .await
        .unwrap()
        .unwrap()
        .version();

    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .event_type_store()
        .find_by_id(et.id(), app.org_id())
        .await
        .unwrap()
        .unwrap();
    loaded.update("versioned-v2".to_string(), None).unwrap();
    uow.event_type_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    let v2 = read_store
        .find_by_id(et.id(), app.org_id())
        .await
        .unwrap()
        .unwrap()
        .version();

    assert_ne!(v1, v2);
}

// ---------------------------------------------------------------------------
// PgEndpointStore + PgUnitOfWork tests
// ---------------------------------------------------------------------------

/// Helper: insert an application and event type, return both (endpoints have FKs to both).
async fn insert_app_and_event_type(
    factory: &PgUnitOfWorkFactory,
) -> (Application, EventType) {
    let org_id = insert_org(factory).await;
    let app = any_application_for_org(&org_id, "ep-parent");
    let et = EventType::new(app.id().clone(), "user.created".to_string(), None).unwrap();
    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.event_type_store().insert(&et).await.unwrap();
    uow.commit().await.unwrap();
    (app, et)
}

fn any_endpoint_for(app: &Application, et: &EventType) -> Endpoint {
    Endpoint::new(
        app.id().clone(),
        format!("https://example.com/webhook/{}", uuid::Uuid::new_v4()),
        format!("whsec_{}", uuid::Uuid::new_v4()),
        vec![et.id().clone()],
    )
    .unwrap()
}

#[tokio::test]
async fn insert_and_find_endpoint() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let (app, et) = insert_app_and_event_type(&factory).await;
    let ep = any_endpoint_for(&app, &et);

    let mut uow = factory.begin().await.unwrap();
    uow.endpoint_store().insert(&ep).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let found = uow.endpoint_store().find_by_id(ep.id(), app.org_id()).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.url(), ep.url());
    assert_eq!(found.app_id(), app.id());
    assert_eq!(found.event_type_ids().len(), 1);
    assert_eq!(found.event_type_ids()[0], *et.id());
}

#[tokio::test]
async fn save_updates_endpoint() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let (app, et) = insert_app_and_event_type(&factory).await;

    // Create a second event type
    let et2 = EventType::new(app.id().clone(), "user.updated".to_string(), None).unwrap();
    {
        let mut uow = factory.begin().await.unwrap();
        uow.event_type_store().insert(&et2).await.unwrap();
        uow.commit().await.unwrap();
    }

    let ep = any_endpoint_for(&app, &et);

    let mut uow = factory.begin().await.unwrap();
    uow.endpoint_store().insert(&ep).await.unwrap();
    uow.commit().await.unwrap();

    // Load, update, save
    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .endpoint_store()
        .find_by_id(ep.id(), app.org_id())
        .await
        .unwrap()
        .unwrap();
    loaded
        .update(
            "https://updated.example.com/webhook".to_string(),
            "whsec_new".to_string(),
            vec![et2.id().clone()],
        )
        .unwrap();
    uow.endpoint_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    // Verify
    let mut uow = factory.begin().await.unwrap();
    let found = uow
        .endpoint_store()
        .find_by_id(ep.id(), app.org_id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.url(), "https://updated.example.com/webhook");
    assert_eq!(found.event_type_ids().len(), 1);
    assert_eq!(found.event_type_ids()[0], *et2.id());
}

#[tokio::test]
async fn delete_removes_endpoint_and_subscriptions() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let (app, et) = insert_app_and_event_type(&factory).await;
    let ep = any_endpoint_for(&app, &et);

    let mut uow = factory.begin().await.unwrap();
    uow.endpoint_store().insert(&ep).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    uow.endpoint_store().delete(ep.id()).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let found = uow.endpoint_store().find_by_id(ep.id(), app.org_id()).await.unwrap();
    assert!(found.is_none());
}

// ---------------------------------------------------------------------------
// PgEndpointReadStore tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn read_store_list_endpoints_by_app() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgEndpointReadStore::new(pool);
    let (app, et) = insert_app_and_event_type(&factory).await;

    for _ in 0..3 {
        let ep = any_endpoint_for(&app, &et);
        let mut uow = factory.begin().await.unwrap();
        uow.endpoint_store().insert(&ep).await.unwrap();
        uow.commit().await.unwrap();
    }

    let listed = read_store.list_by_app(app.id(), app.org_id(), 0, 10).await.unwrap();
    assert_eq!(listed.len(), 3);
    for i in 0..listed.len() - 1 {
        assert!(listed[i].created_at() >= listed[i + 1].created_at());
    }
}

#[tokio::test]
async fn read_store_count_endpoints_by_app() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgEndpointReadStore::new(pool);
    let (app, et) = insert_app_and_event_type(&factory).await;

    for _ in 0..3 {
        let ep = any_endpoint_for(&app, &et);
        let mut uow = factory.begin().await.unwrap();
        uow.endpoint_store().insert(&ep).await.unwrap();
        uow.commit().await.unwrap();
    }

    let count = read_store.count_by_app(app.id(), app.org_id()).await.unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn read_store_find_endpoint_by_id() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgEndpointReadStore::new(pool);
    let (app, et) = insert_app_and_event_type(&factory).await;
    let ep = any_endpoint_for(&app, &et);

    let mut uow = factory.begin().await.unwrap();
    uow.endpoint_store().insert(&ep).await.unwrap();
    uow.commit().await.unwrap();

    let found = read_store.find_by_id(ep.id(), app.org_id()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().url(), ep.url());
}

// ---------------------------------------------------------------------------
// PgOrganizationStore + PgUnitOfWork tests
// ---------------------------------------------------------------------------

fn any_organization(name: &str, slug: &str) -> Organization {
    Organization::new(name.to_string(), slug.to_string()).unwrap()
}

#[tokio::test]
async fn org_insert_and_find_by_id() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org = any_organization("test-org", "test-org");

    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().insert(&org).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let found = uow.organization_store().find_by_id(org.id()).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name(), "test-org");
    assert_eq!(found.slug(), "test-org");
}

#[tokio::test]
async fn org_find_by_id_returns_none_for_nonexistent() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);

    let mut uow = factory.begin().await.unwrap();
    let result = uow
        .organization_store()
        .find_by_id(&OrganizationId::new())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn org_save_updates_organization() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org = any_organization("original", "original-slug");

    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().insert(&org).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .organization_store()
        .find_by_id(org.id())
        .await
        .unwrap()
        .unwrap();
    loaded.rename("renamed".to_string()).unwrap();
    uow.organization_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let found = uow
        .organization_store()
        .find_by_id(org.id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.name(), "renamed");
    // slug should be unchanged
    assert_eq!(found.slug(), "original-slug");
}

#[tokio::test]
async fn org_save_with_stale_version_returns_conflict() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org = any_organization("org", "org-slug");

    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().insert(&org).await.unwrap();
    uow.commit().await.unwrap();

    // Update to advance version
    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .organization_store()
        .find_by_id(org.id())
        .await
        .unwrap()
        .unwrap();
    loaded.rename("updated".to_string()).unwrap();
    uow.organization_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    // Try to save with stale version
    let mut stale = org.clone();
    stale.rename("stale-update".to_string()).unwrap();
    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().save(&stale).await.unwrap();
    let result = uow.commit().await;

    assert!(matches!(result, Err(ApplicationError::Conflict)));
}

#[tokio::test]
async fn org_delete_removes_organization() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org = any_organization("to-delete", "to-delete-slug");

    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().insert(&org).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().delete(org.id()).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = factory.begin().await.unwrap();
    let found = uow.organization_store().find_by_id(org.id()).await.unwrap();
    assert!(found.is_none());
}

// ---------------------------------------------------------------------------
// PgOrganizationReadStore tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn org_read_store_find_by_id_returns_organization() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOrganizationReadStore::new(pool);
    let org = any_organization("readable", "readable-slug");

    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().insert(&org).await.unwrap();
    uow.commit().await.unwrap();

    let found = read_store.find_by_id(org.id()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), "readable");
}

#[tokio::test]
async fn org_read_store_find_by_id_returns_none() {
    let (pool, _container) = setup().await;
    let read_store = PgOrganizationReadStore::new(pool);

    let result = read_store.find_by_id(&OrganizationId::new()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn org_read_store_list_returns_organizations_ordered_by_created_at_desc() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOrganizationReadStore::new(pool);

    let orgs: Vec<Organization> = (0..3)
        .map(|i| any_organization(&format!("org-{i}"), &format!("org-slug-{i}")))
        .collect();

    for org in &orgs {
        let mut uow = factory.begin().await.unwrap();
        uow.organization_store().insert(org).await.unwrap();
        uow.commit().await.unwrap();
    }

    let listed = read_store.list(0, 10).await.unwrap();
    assert_eq!(listed.len(), 3);
    for i in 0..listed.len() - 1 {
        assert!(listed[i].created_at() >= listed[i + 1].created_at());
    }
}

#[tokio::test]
async fn org_read_store_list_respects_offset_and_limit() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOrganizationReadStore::new(pool);

    for i in 0..5 {
        let org = any_organization(&format!("paged-{i}"), &format!("paged-slug-{i}"));
        let mut uow = factory.begin().await.unwrap();
        uow.organization_store().insert(&org).await.unwrap();
        uow.commit().await.unwrap();
    }

    let page = read_store.list(2, 2).await.unwrap();
    assert_eq!(page.len(), 2);
}

#[tokio::test]
async fn org_read_store_count_returns_total() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOrganizationReadStore::new(pool);

    for i in 0..3 {
        let org = any_organization(&format!("counted-{i}"), &format!("counted-slug-{i}"));
        let mut uow = factory.begin().await.unwrap();
        uow.organization_store().insert(&org).await.unwrap();
        uow.commit().await.unwrap();
    }

    let count = read_store.count().await.unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn org_version_changes_after_update() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOrganizationReadStore::new(pool);
    let org = any_organization("versioned", "versioned-slug");

    let mut uow = factory.begin().await.unwrap();
    uow.organization_store().insert(&org).await.unwrap();
    uow.commit().await.unwrap();

    let v1 = read_store
        .find_by_id(org.id())
        .await
        .unwrap()
        .unwrap()
        .version();

    let mut uow = factory.begin().await.unwrap();
    let mut loaded = uow
        .organization_store()
        .find_by_id(org.id())
        .await
        .unwrap()
        .unwrap();
    loaded.rename("versioned-v2".to_string()).unwrap();
    uow.organization_store().save(&loaded).await.unwrap();
    uow.commit().await.unwrap();

    let v2 = read_store
        .find_by_id(org.id())
        .await
        .unwrap()
        .unwrap()
        .version();

    assert_ne!(v1, v2);
}

// ---------------------------------------------------------------------------
// SendMessage integration tests (message + attempts via UoW)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_message_and_attempts_via_uow() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());

    // Setup: org -> app -> event_type -> 2 endpoints with subscriptions
    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "msg-app");
    let et = EventType::new(app.id().clone(), "user.created".to_string(), None).unwrap();

    let ep1 = Endpoint::new(
        app.id().clone(),
        format!("https://a.example.com/hook/{}", uuid::Uuid::new_v4()),
        format!("whsec_{}", uuid::Uuid::new_v4()),
        vec![et.id().clone()],
    )
    .unwrap();
    let ep2 = Endpoint::new(
        app.id().clone(),
        format!("https://b.example.com/hook/{}", uuid::Uuid::new_v4()),
        format!("whsec_{}", uuid::Uuid::new_v4()),
        vec![et.id().clone()],
    )
    .unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app).await.unwrap();
        uow.event_type_store().insert(&et).await.unwrap();
        uow.endpoint_store().insert(&ep1).await.unwrap();
        uow.endpoint_store().insert(&ep2).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Insert message + 2 attempts in one UoW commit
    let msg = pigeon_domain::message::Message::new(
        app.id().clone(),
        et.id().clone(),
        serde_json::json!({"user": "u1"}),
        Some("test-key-1".into()),
        chrono::Duration::hours(24),
    )
    .unwrap();

    let att1 = pigeon_domain::attempt::Attempt::new(
        msg.id().clone(),
        ep1.id().clone(),
        chrono::Utc::now(),
    );
    let att2 = pigeon_domain::attempt::Attempt::new(
        msg.id().clone(),
        ep2.id().clone(),
        chrono::Utc::now(),
    );

    {
        let mut uow = factory.begin().await.unwrap();
        let result = uow.message_store().insert_or_get_existing(&msg, app.org_id()).await.unwrap();
        assert!(!result.was_existing);
        uow.attempt_store().insert(&att1).await.unwrap();
        uow.attempt_store().insert(&att2).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Verify message exists via a direct lookup (insert_or_get_existing with same key)
    {
        let mut uow = factory.begin().await.unwrap();
        let result = uow.message_store().insert_or_get_existing(&msg, app.org_id()).await.unwrap();
        assert!(result.was_existing);
        assert_eq!(*result.message.id(), *msg.id());
        uow.commit().await.unwrap();
    }
}

#[tokio::test]
async fn message_idempotency_returns_existing() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);

    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "idem-app");
    let et = EventType::new(app.id().clone(), "user.created".to_string(), None).unwrap();
    {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app).await.unwrap();
        uow.event_type_store().insert(&et).await.unwrap();
        uow.commit().await.unwrap();
    }

    let msg = pigeon_domain::message::Message::new(
        app.id().clone(),
        et.id().clone(),
        serde_json::json!({"user": "u1"}),
        Some("idem-key-1".into()),
        chrono::Duration::hours(24),
    )
    .unwrap();

    // First insert
    {
        let mut uow = factory.begin().await.unwrap();
        let result = uow.message_store().insert_or_get_existing(&msg, app.org_id()).await.unwrap();
        assert!(!result.was_existing);
        uow.commit().await.unwrap();
    }

    // Second insert with same idempotency key -> was_existing=true
    let msg2 = pigeon_domain::message::Message::new(
        app.id().clone(),
        et.id().clone(),
        serde_json::json!({"user": "u2"}),
        Some("idem-key-1".into()),
        chrono::Duration::hours(24),
    )
    .unwrap();
    {
        let mut uow = factory.begin().await.unwrap();
        let result = uow.message_store().insert_or_get_existing(&msg2, app.org_id()).await.unwrap();
        assert!(result.was_existing);
        assert_eq!(*result.message.id(), *msg.id());
        uow.commit().await.unwrap();
    }
}

#[tokio::test]
async fn endpoint_read_store_finds_enabled_endpoints_by_event_type() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgEndpointReadStore::new(pool);

    let org_id = insert_org(&factory).await;
    let app = any_application_for_org(&org_id, "ep-filter-app");
    let et = EventType::new(app.id().clone(), "order.placed".to_string(), None).unwrap();

    let ep_enabled_1 = Endpoint::new(
        app.id().clone(),
        format!("https://a.example.com/{}", uuid::Uuid::new_v4()),
        format!("whsec_{}", uuid::Uuid::new_v4()),
        vec![et.id().clone()],
    )
    .unwrap();
    let ep_enabled_2 = Endpoint::new(
        app.id().clone(),
        format!("https://b.example.com/{}", uuid::Uuid::new_v4()),
        format!("whsec_{}", uuid::Uuid::new_v4()),
        vec![et.id().clone()],
    )
    .unwrap();
    let mut ep_disabled = Endpoint::new(
        app.id().clone(),
        format!("https://c.example.com/{}", uuid::Uuid::new_v4()),
        format!("whsec_{}", uuid::Uuid::new_v4()),
        vec![et.id().clone()],
    )
    .unwrap();
    ep_disabled.disable();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app).await.unwrap();
        uow.event_type_store().insert(&et).await.unwrap();
        uow.endpoint_store().insert(&ep_enabled_1).await.unwrap();
        uow.endpoint_store().insert(&ep_enabled_2).await.unwrap();
        uow.endpoint_store().insert(&ep_disabled).await.unwrap();
        uow.commit().await.unwrap();
    }

    let found = read_store
        .find_enabled_by_app_and_event_type(app.id(), et.id(), app.org_id())
        .await
        .unwrap();

    assert_eq!(found.len(), 2);
    // All returned endpoints should be enabled
    for ep in &found {
        assert!(ep.enabled());
    }
}

// ---------------------------------------------------------------------------
// PgOidcConfigStore + PgOidcConfigReadStore tests
// ---------------------------------------------------------------------------

fn any_oidc_config_for_org(org_id: &OrganizationId) -> OidcConfig {
    OidcConfig::new(
        org_id.clone(),
        format!("https://auth-{}.example.com", uuid::Uuid::new_v4().simple()),
        format!("api-{}", uuid::Uuid::new_v4().simple()),
        format!(
            "https://auth-{}.example.com/.well-known/jwks.json",
            uuid::Uuid::new_v4().simple()
        ),
    )
    .unwrap()
}

#[tokio::test]
async fn insert_and_find_oidc_config() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOidcConfigReadStore::new(pool);
    let org_id = insert_org(&factory).await;
    let config = any_oidc_config_for_org(&org_id);

    {
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config).await.unwrap();
        uow.commit().await.unwrap();
    }

    let found = read_store.find_by_id(config.id()).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.issuer_url(), config.issuer_url());
    assert_eq!(found.audience(), config.audience());
    assert_eq!(found.jwks_url(), config.jwks_url());
    assert_eq!(found.org_id(), config.org_id());
}

#[tokio::test]
async fn find_oidc_config_by_issuer_and_audience() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOidcConfigReadStore::new(pool);
    let org_id = insert_org(&factory).await;
    let config = any_oidc_config_for_org(&org_id);

    {
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config).await.unwrap();
        uow.commit().await.unwrap();
    }

    let found = read_store
        .find_by_issuer_and_audience(config.issuer_url(), config.audience())
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(*found.unwrap().id(), *config.id());
}

#[tokio::test]
async fn delete_oidc_config() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOidcConfigReadStore::new(pool);
    let org_id = insert_org(&factory).await;
    let config = any_oidc_config_for_org(&org_id);

    {
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Delete
    {
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().delete(config.id()).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Verify gone
    let found = read_store.find_by_id(config.id()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn oidc_config_unique_issuer_audience() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool);
    let org_id = insert_org(&factory).await;

    let issuer = format!("https://auth-{}.example.com", uuid::Uuid::new_v4().simple());
    let audience = format!("api-{}", uuid::Uuid::new_v4().simple());

    let config1 = OidcConfig::new(
        org_id.clone(),
        issuer.clone(),
        audience.clone(),
        format!("{issuer}/.well-known/jwks.json"),
    )
    .unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config1).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Second config with same (issuer, audience) -> should fail on commit
    let config2 = OidcConfig::new(
        org_id,
        issuer.clone(),
        audience.clone(),
        format!("{issuer}/.well-known/jwks2.json"),
    )
    .unwrap();

    let mut uow = factory.begin().await.unwrap();
    uow.oidc_config_store().insert(&config2).await.unwrap();
    let result = uow.commit().await;

    // The unique constraint violation should surface as an error
    assert!(result.is_err(), "expected unique constraint error, got Ok");
}

#[tokio::test]
async fn list_oidc_configs_by_org() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgOidcConfigReadStore::new(pool);
    let org_a = insert_org(&factory).await;
    let org_b = insert_org(&factory).await;

    // 2 configs for org_a
    for _ in 0..2 {
        let config = any_oidc_config_for_org(&org_a);
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config).await.unwrap();
        uow.commit().await.unwrap();
    }

    // 1 config for org_b
    {
        let config = any_oidc_config_for_org(&org_b);
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config).await.unwrap();
        uow.commit().await.unwrap();
    }

    let configs_a = read_store.list_by_org(&org_a, 0, 10).await.unwrap();
    assert_eq!(configs_a.len(), 2);

    let count_a = read_store.count_by_org(&org_a).await.unwrap();
    assert_eq!(count_a, 2);

    let configs_b = read_store.list_by_org(&org_b, 0, 10).await.unwrap();
    assert_eq!(configs_b.len(), 1);
}

// ---------------------------------------------------------------------------
// PgDeliveryQueue tests
// ---------------------------------------------------------------------------

/// Seed the full chain: org → app → event_type → endpoint → message → attempt.
/// Returns (attempt, endpoint, message, app) for use in delivery queue tests.
async fn seed_pending_attempt(
    factory: &PgUnitOfWorkFactory,
) -> (Attempt, Endpoint, pigeon_domain::message::Message, Application) {
    let org_id = insert_org(factory).await;
    let app = any_application_for_org(&org_id, "delivery-app");

    let mut uow = factory.begin().await.unwrap();
    uow.application_store().insert(&app).await.unwrap();
    uow.commit().await.unwrap();

    let et = EventType::new(app.id().clone(), "test.event".into(), None).unwrap();
    let mut uow = factory.begin().await.unwrap();
    uow.event_type_store().insert(&et).await.unwrap();
    uow.commit().await.unwrap();

    let ep = Endpoint::new(
        app.id().clone(),
        "https://example.com/hook".into(),
        "whsec_test123".into(),
        vec![et.id().clone()],
    )
    .unwrap();
    let mut uow = factory.begin().await.unwrap();
    uow.endpoint_store().insert(&ep).await.unwrap();
    uow.commit().await.unwrap();

    let msg = pigeon_domain::message::Message::new(
        app.id().clone(),
        et.id().clone(),
        serde_json::json!({"hello": "world"}),
        None,
        chrono::Duration::hours(24),
    )
    .unwrap();
    let mut uow = factory.begin().await.unwrap();
    uow.message_store()
        .insert_or_get_existing(&msg, &org_id)
        .await
        .unwrap();
    uow.commit().await.unwrap();

    let attempt = Attempt::new(msg.id().clone(), ep.id().clone(), chrono::Utc::now());
    let mut uow = factory.begin().await.unwrap();
    uow.attempt_store().insert(&attempt).await.unwrap();
    uow.commit().await.unwrap();

    (attempt, ep, msg, app)
}

#[tokio::test]
async fn delivery_queue_dequeue_returns_pending_attempts() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let queue = PgDeliveryQueue::new(pool);

    let (_attempt, ep, msg, _app) = seed_pending_attempt(&factory).await;

    let tasks = queue.dequeue(10).await.unwrap();
    assert_eq!(tasks.len(), 1);

    let task = &tasks[0];
    assert_eq!(task.endpoint_url, "https://example.com/hook");
    assert_eq!(task.signing_secret, "whsec_test123");
    assert_eq!(task.payload, serde_json::json!({"hello": "world"}));
    // attempt_number should be bumped from 1 to 2 by dequeue
    assert_eq!(task.attempt_number, 2);
    assert_eq!(*task.endpoint_id.as_uuid(), *ep.id().as_uuid());
    assert_eq!(*task.message_id.as_uuid(), *msg.id().as_uuid());
}

#[tokio::test]
async fn delivery_queue_dequeue_skips_already_dequeued() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let queue = PgDeliveryQueue::new(pool);

    seed_pending_attempt(&factory).await;

    // First dequeue claims the attempt
    let tasks = queue.dequeue(10).await.unwrap();
    assert_eq!(tasks.len(), 1);

    // Second dequeue should find nothing (status is now in_flight)
    let tasks = queue.dequeue(10).await.unwrap();
    assert_eq!(tasks.len(), 0);
}

#[tokio::test]
async fn delivery_queue_record_success() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let queue = PgDeliveryQueue::new(pool.clone());

    seed_pending_attempt(&factory).await;

    let tasks = queue.dequeue(10).await.unwrap();
    let task = &tasks[0];

    queue
        .record_success(&task.attempt_id, 200, "OK".into(), 150)
        .await
        .unwrap();

    // Verify the attempt is no longer dequeue-able
    let tasks = queue.dequeue(10).await.unwrap();
    assert_eq!(tasks.len(), 0);

    // Verify status in DB
    let row = sqlx::query_as::<_, (String, Option<i16>, Option<i64>)>(
        "SELECT status, response_code, duration_ms FROM attempts WHERE id = $1",
    )
    .bind(task.attempt_id.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "succeeded");
    assert_eq!(row.1, Some(200));
    assert_eq!(row.2, Some(150));
}

#[tokio::test]
async fn delivery_queue_record_failure_with_retry() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let queue = PgDeliveryQueue::new(pool.clone());

    seed_pending_attempt(&factory).await;

    let tasks = queue.dequeue(10).await.unwrap();
    let task = &tasks[0];

    let next = chrono::Utc::now() + chrono::Duration::seconds(60);
    queue
        .record_failure(
            &task.attempt_id,
            Some(500),
            Some("Internal Server Error".into()),
            42,
            Some(next),
        )
        .await
        .unwrap();

    // Should be dequeue-able again after next_attempt_at passes
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT status FROM attempts WHERE id = $1",
    )
    .bind(task.attempt_id.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "pending");
}

#[tokio::test]
async fn delivery_queue_record_failure_final() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let queue = PgDeliveryQueue::new(pool.clone());

    seed_pending_attempt(&factory).await;

    let tasks = queue.dequeue(10).await.unwrap();
    let task = &tasks[0];

    queue
        .record_failure(&task.attempt_id, Some(500), Some("fail".into()), 10, None)
        .await
        .unwrap();

    let row = sqlx::query_as::<_, (String,)>(
        "SELECT status FROM attempts WHERE id = $1",
    )
    .bind(task.attempt_id.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "failed");
}

#[tokio::test]
async fn delivery_queue_insert_dead_letter() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let queue = PgDeliveryQueue::new(pool.clone());

    let (_attempt, ep, msg, app) = seed_pending_attempt(&factory).await;

    queue
        .insert_dead_letter(
            ep.id(),
            msg.id(),
            app.id(),
            Some(503),
            Some("Service Unavailable".into()),
        )
        .await
        .unwrap();

    let row = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM dead_letters WHERE endpoint_id = $1 AND message_id = $2",
    )
    .bind(ep.id().as_uuid())
    .bind(msg.id().as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, 1);
}

#[tokio::test]
async fn delivery_queue_empty_returns_empty_vec() {
    let (pool, _container) = setup().await;
    let queue = PgDeliveryQueue::new(pool);

    let tasks = queue.dequeue(10).await.unwrap();
    assert!(tasks.is_empty());
}

// ---------------------------------------------------------------------------
// Cross-tenant SQL isolation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cross_tenant_application_isolation() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let read_store = PgApplicationReadStore::new(pool.clone());

    let org_a = insert_org(&factory).await;
    let org_b = insert_org(&factory).await;

    let app_a = any_application_for_org(&org_a, "app-a");
    let app_b = any_application_for_org(&org_b, "app-b");

    {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app_a).await.unwrap();
        uow.application_store().insert(&app_b).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Org A can see its own app
    let found = read_store.find_by_id(app_a.id()).await.unwrap();
    assert!(found.is_some());

    // Org A's list only returns its own apps
    let list_a = read_store.list_by_org(&org_a, 0, 100).await.unwrap();
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_a[0].name(), "app-a");

    // Org B's list only returns its own apps
    let list_b = read_store.list_by_org(&org_b, 0, 100).await.unwrap();
    assert_eq!(list_b.len(), 1);
    assert_eq!(list_b[0].name(), "app-b");
}

#[tokio::test]
async fn cross_tenant_endpoint_isolation() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let ep_read_store = PgEndpointReadStore::new(pool.clone());

    let org_a = insert_org(&factory).await;
    let org_b = insert_org(&factory).await;

    let app_a = any_application_for_org(&org_a, "app-a");
    let app_b = any_application_for_org(&org_b, "app-b");

    {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app_a).await.unwrap();
        uow.application_store().insert(&app_b).await.unwrap();
        uow.commit().await.unwrap();
    }

    let et_a = EventType::new(app_a.id().clone(), "test.event".into(), None).unwrap();
    let et_b = EventType::new(app_b.id().clone(), "test.event".into(), None).unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.event_type_store().insert(&et_a).await.unwrap();
        uow.event_type_store().insert(&et_b).await.unwrap();
        uow.commit().await.unwrap();
    }

    let ep_a = Endpoint::new(
        app_a.id().clone(),
        "https://a.com/hook".into(),
        "whsec_a".into(),
        vec![et_a.id().clone()],
    )
    .unwrap();

    let ep_b = Endpoint::new(
        app_b.id().clone(),
        "https://b.com/hook".into(),
        "whsec_b".into(),
        vec![et_b.id().clone()],
    )
    .unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.endpoint_store().insert(&ep_a).await.unwrap();
        uow.endpoint_store().insert(&ep_b).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Org A can find its own endpoint
    let found = ep_read_store.find_by_id(ep_a.id(), &org_a).await.unwrap();
    assert!(found.is_some());

    // Org A cannot find org B's endpoint
    let cross = ep_read_store.find_by_id(ep_b.id(), &org_a).await.unwrap();
    assert!(cross.is_none());

    // Org A's list only returns its own endpoints
    let list_a = ep_read_store
        .list_by_app(app_a.id(), &org_a, 0, 100)
        .await
        .unwrap();
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_a[0].url(), "https://a.com/hook");

    // Org A cannot list org B's app's endpoints
    let cross_list = ep_read_store
        .list_by_app(app_b.id(), &org_a, 0, 100)
        .await
        .unwrap();
    assert!(cross_list.is_empty());
}

#[tokio::test]
async fn cross_tenant_event_type_isolation() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let et_read_store = PgEventTypeReadStore::new(pool.clone());

    let org_a = insert_org(&factory).await;
    let org_b = insert_org(&factory).await;

    let app_a = any_application_for_org(&org_a, "app-a");
    let app_b = any_application_for_org(&org_b, "app-b");

    {
        let mut uow = factory.begin().await.unwrap();
        uow.application_store().insert(&app_a).await.unwrap();
        uow.application_store().insert(&app_b).await.unwrap();
        uow.commit().await.unwrap();
    }

    let et_a = EventType::new(app_a.id().clone(), "a.event".into(), None).unwrap();
    let et_b = EventType::new(app_b.id().clone(), "b.event".into(), None).unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.event_type_store().insert(&et_a).await.unwrap();
        uow.event_type_store().insert(&et_b).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Org A can find its own event type
    let found = et_read_store
        .find_by_id(et_a.id(), &org_a)
        .await
        .unwrap();
    assert!(found.is_some());

    // Org A cannot find org B's event type
    let cross = et_read_store
        .find_by_id(et_b.id(), &org_a)
        .await
        .unwrap();
    assert!(cross.is_none());

    // Org A's list only returns its own event types
    let list_a = et_read_store
        .list_by_app(app_a.id(), &org_a, 0, 100)
        .await
        .unwrap();
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_a[0].name(), "a.event");
}

#[tokio::test]
async fn cross_tenant_oidc_config_isolation() {
    let (pool, _container) = setup().await;
    let factory = PgUnitOfWorkFactory::new(pool.clone());
    let oidc_read_store = PgOidcConfigReadStore::new(pool.clone());

    let org_a = insert_org(&factory).await;
    let org_b = insert_org(&factory).await;

    let config_a = OidcConfig::new(
        org_a.clone(),
        "https://auth-a.example.com".into(),
        "audience-a".into(),
        "https://auth-a.example.com/.well-known/jwks.json".into(),
    )
    .unwrap();

    let config_b = OidcConfig::new(
        org_b.clone(),
        "https://auth-b.example.com".into(),
        "audience-b".into(),
        "https://auth-b.example.com/.well-known/jwks.json".into(),
    )
    .unwrap();

    {
        let mut uow = factory.begin().await.unwrap();
        uow.oidc_config_store().insert(&config_a).await.unwrap();
        uow.oidc_config_store().insert(&config_b).await.unwrap();
        uow.commit().await.unwrap();
    }

    // Org A's configs don't include org B's
    let list_a = oidc_read_store.list_by_org(&org_a, 0, 100).await.unwrap();
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_a[0].issuer_url(), "https://auth-a.example.com");

    let count_a = oidc_read_store.count_by_org(&org_a).await.unwrap();
    assert_eq!(count_a, 1);

    // Org B's configs don't include org A's
    let list_b = oidc_read_store.list_by_org(&org_b, 0, 100).await.unwrap();
    assert_eq!(list_b.len(), 1);
    assert_eq!(list_b[0].issuer_url(), "https://auth-b.example.com");
}
