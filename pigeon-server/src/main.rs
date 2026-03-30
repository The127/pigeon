use std::sync::Arc;

use anyhow::Context;
use chrono::Duration;
use clap::{Parser, Subcommand};
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::PgPool;
use tokio::sync::watch;
use pigeon_application::ports::stores::OrganizationReadStore;
use tracing::{info, warn};

use pigeon_api::auth::CachedJwksProvider;
use pigeon_api::state::AppState;
use pigeon_application::commands::create_application::CreateApplicationHandler;
use pigeon_application::commands::create_endpoint::CreateEndpointHandler;
use pigeon_application::commands::create_event_type::CreateEventTypeHandler;
use pigeon_application::commands::create_oidc_config::CreateOidcConfigHandler;
use pigeon_application::commands::create_organization::CreateOrganizationHandler;
use pigeon_application::commands::delete_application::DeleteApplicationHandler;
use pigeon_application::commands::delete_endpoint::DeleteEndpointHandler;
use pigeon_application::commands::delete_event_type::DeleteEventTypeHandler;
use pigeon_application::commands::delete_oidc_config::DeleteOidcConfigHandler;
use pigeon_application::commands::delete_organization::DeleteOrganizationHandler;
use pigeon_application::commands::replay_dead_letter::ReplayDeadLetterHandler;
use pigeon_application::commands::retrigger_message::RetriggerMessageHandler;
use pigeon_application::commands::retry_attempt::RetryAttemptHandler;
use pigeon_application::commands::send_message::SendMessageHandler;
use pigeon_application::commands::send_test_event::SendTestEventHandler;
use pigeon_application::commands::update_application::UpdateApplicationHandler;
use pigeon_application::commands::update_endpoint::UpdateEndpointHandler;
use pigeon_application::commands::update_event_type::UpdateEventTypeHandler;
use pigeon_application::commands::update_organization::UpdateOrganizationHandler;
use pigeon_application::queries::get_app_stats::GetAppStatsHandler;
use pigeon_application::queries::get_endpoint_stats::GetEndpointStatsHandler;
use pigeon_application::queries::get_event_type_stats::GetEventTypeStatsHandler;
use pigeon_application::queries::get_application_by_id::GetApplicationByIdHandler;
use pigeon_application::queries::get_dead_letter_by_id::GetDeadLetterByIdHandler;
use pigeon_application::queries::get_message_by_id::GetMessageByIdHandler;
use pigeon_application::queries::list_attempts_by_message::ListAttemptsByMessageHandler;
use pigeon_application::queries::list_dead_letters_by_app::ListDeadLettersByAppHandler;
use pigeon_application::queries::list_messages_by_app::ListMessagesByAppHandler;
use pigeon_application::queries::get_endpoint_by_id::GetEndpointByIdHandler;
use pigeon_application::queries::get_event_type_by_id::GetEventTypeByIdHandler;
use pigeon_application::queries::get_oidc_config_by_id::GetOidcConfigByIdHandler;
use pigeon_application::queries::get_organization_by_id::GetOrganizationByIdHandler;
use pigeon_application::queries::list_applications::ListApplicationsHandler;
use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByAppHandler;
use pigeon_application::queries::list_event_types_by_app::ListEventTypesByAppHandler;
use pigeon_application::queries::list_oidc_configs_by_org::ListOidcConfigsByOrgHandler;
use pigeon_application::queries::list_audit_log::ListAuditLogHandler;
use pigeon_application::queries::list_organizations::ListOrganizationsHandler;
use pigeon_application::commands::disable_endpoint::DisableEndpointHandler;
use pigeon_application::services::auto_disable_saga::AutoDisableEndpointSaga;
use pigeon_application::services::delivery_projection::DeliveryProjectionSubscriber;
use pigeon_application::services::delivery_worker::{DeliveryWorkerConfig, DeliveryWorkerService};
use pigeon_application::services::outbox_worker::{
    EventSubscriber, LogEventSubscriber, OutboxWorkerConfig, OutboxWorkerService,
};
use pigeon_infrastructure::http::ReqwestWebhookClient;
use pigeon_infrastructure::persistence::{
    PgApplicationReadStore, PgAttemptReadStore, PgAuditReadStore, PgAuditStore, PgDeadLetterReadStore, PgDeliveryQueue,
    PgEndpointReadStore, PgEndpointStatsReadStore, PgEventOutbox, PgEventTypeReadStore,
    PgHealthChecker, PgMessageReadStore, PgOidcConfigReadStore, PgOrganizationReadStore,
    PgProjectionStore, PgEventTypeStatsReadStore, PgStatsReadStore, PgUnitOfWorkFactory,
};

mod bootstrap;
mod config;

use crate::bootstrap::bootstrap_organization;
use crate::config::PigeonConfig;

#[derive(Parser)]
#[command(name = "pigeon", about = "Pigeon webhook service")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the API and worker together
    Serve,
    /// Run only the API server
    Api,
    /// Run only the delivery worker
    Worker,
    /// Run database migrations
    Migrate,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let config = PigeonConfig::from_env()?;

    // Install Prometheus metrics recorder (global, used by all crates via metrics facade)
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .context("Failed to install Prometheus metrics recorder")?;
    let metrics_render: Arc<dyn Fn() -> String + Send + Sync> = {
        let handle = prometheus_handle;
        Arc::new(move || handle.render())
    };

    match cli.command {
        Commands::Serve => {
            let pool = create_pool(&config).await?;
            run_migrations(&pool).await?;
            run_bootstrap(&pool, &config).await?;
            let admin_org_id = resolve_admin_org(&pool, &config).await?;

            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            let api_pool = pool.clone();
            let api_config = config.listen_addr.clone();
            let jwks_ttl = config.jwks_cache_ttl;
            let api_shutdown_rx = shutdown_rx.clone();
            let api_metrics = metrics_render.clone();

            let api_handle = tokio::spawn(async move {
                run_api(api_pool, &api_config, jwks_ttl, admin_org_id, api_shutdown_rx, api_metrics).await
            });

            let worker = create_worker(&pool, &config);
            let worker_shutdown = shutdown_rx.clone();
            let worker_handle = tokio::spawn(async move { worker.run(worker_shutdown).await });

            let outbox = create_outbox_worker(&pool, &config);
            let outbox_handle = tokio::spawn(async move { outbox.run(shutdown_rx).await });

            tokio::signal::ctrl_c()
                .await
                .context("Failed to listen for ctrl-c")?;
            info!("Shutdown signal received");
            let _ = shutdown_tx.send(true);

            let (api_result, _, _) = tokio::join!(api_handle, worker_handle, outbox_handle);
            api_result??;
        }
        Commands::Api => {
            let pool = create_pool(&config).await?;
            run_bootstrap(&pool, &config).await?;

            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            let admin_org_id = resolve_admin_org(&pool, &config).await?;

            let api_handle = tokio::spawn(async move {
                run_api(pool, &config.listen_addr, config.jwks_cache_ttl, admin_org_id, shutdown_rx, metrics_render).await
            });

            tokio::signal::ctrl_c()
                .await
                .context("Failed to listen for ctrl-c")?;
            info!("Shutdown signal received");
            let _ = shutdown_tx.send(true);

            api_handle.await??;
        }
        Commands::Worker => {
            let pool = create_pool(&config).await?;

            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            let worker = create_worker(&pool, &config);
            let worker_shutdown = shutdown_rx.clone();
            let worker_handle = tokio::spawn(async move { worker.run(worker_shutdown).await });

            let outbox = create_outbox_worker(&pool, &config);
            let outbox_handle = tokio::spawn(async move { outbox.run(shutdown_rx).await });

            tokio::signal::ctrl_c()
                .await
                .context("Failed to listen for ctrl-c")?;
            info!("Shutdown signal received");
            let _ = shutdown_tx.send(true);

            let (_, _) = tokio::join!(worker_handle, outbox_handle);
        }
        Commands::Migrate => {
            let pool = create_pool(&config).await?;
            run_migrations(&pool).await?;
            info!("Migrations completed successfully");
        }
    }

    Ok(())
}

async fn create_pool(config: &PigeonConfig) -> anyhow::Result<PgPool> {
    PgPool::connect(&config.database_url)
        .await
        .context("Failed to connect to database")
}

async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../pigeon-infrastructure/migrations")
        .run(pool)
        .await
        .context("Failed to run database migrations")?;

    Ok(())
}

async fn run_bootstrap(pool: &PgPool, config: &PigeonConfig) -> anyhow::Result<()> {
    let uow_factory = PgUnitOfWorkFactory::new(pool.clone());
    let org_read_store = PgOrganizationReadStore::new(pool.clone());
    bootstrap_organization(&uow_factory, &org_read_store, config).await
}

fn create_worker(pool: &PgPool, config: &PigeonConfig) -> DeliveryWorkerService {
    let queue = Arc::new(PgDeliveryQueue::new(pool.clone()));
    let http_client = Arc::new(ReqwestWebhookClient::new(config.worker_http_timeout));

    let worker_config = DeliveryWorkerConfig {
        batch_size: config.worker_batch_size,
        poll_interval: config.worker_poll_interval,
        max_retries: config.worker_max_retries,
        backoff_base_secs: config.worker_backoff_base_secs,
        max_backoff_secs: config.worker_max_backoff_secs,
        cleanup_interval: std::time::Duration::from_secs(config.worker_cleanup_interval_secs),
    };

    DeliveryWorkerService::new(queue, http_client, worker_config)
}

fn create_outbox_worker(pool: &PgPool, config: &PigeonConfig) -> OutboxWorkerService {
    let outbox = Arc::new(PgEventOutbox::new(pool.clone()));
    let dead_letter_read_store = Arc::new(PgDeadLetterReadStore::new(pool.clone()));
    let uow_factory = Arc::new(PgUnitOfWorkFactory::new(pool.clone()));
    let projection_store = Arc::new(PgProjectionStore::new(pool.clone()));

    let subscribers: Vec<Arc<dyn EventSubscriber>> = vec![
        Arc::new(LogEventSubscriber),
        Arc::new(AutoDisableEndpointSaga::new(
            dead_letter_read_store,
            Arc::new(DisableEndpointHandler::new(uow_factory)),
            config.worker_auto_disable_threshold,
        )),
        Arc::new(DeliveryProjectionSubscriber::new(projection_store)),
    ];

    let outbox_config = OutboxWorkerConfig {
        poll_interval: config.worker_poll_interval,
        batch_size: 50,
    };
    OutboxWorkerService::new(outbox, subscribers, outbox_config)
}

/// Look up the bootstrap organization by slug if bootstrap is enabled.
async fn resolve_admin_org(
    pool: &PgPool,
    config: &PigeonConfig,
) -> anyhow::Result<Option<pigeon_domain::organization::OrganizationId>> {
    if !config.bootstrap_org_enabled {
        return Ok(None);
    }

    let org_read_store = PgOrganizationReadStore::new(pool.clone());
    let org = org_read_store
        .find_by_slug(&config.bootstrap_org_slug)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to look up admin org: {e}"))?;

    match org {
        Some(o) => {
            info!(
                org_id = %o.id().as_uuid(),
                slug = %o.slug(),
                "Admin org resolved"
            );
            Ok(Some(o.id().clone()))
        }
        None => {
            warn!(
                slug = %config.bootstrap_org_slug,
                "Bootstrap org slug configured but org not found — admin API will be disabled"
            );
            Ok(None)
        }
    }
}

async fn run_api(
    pool: PgPool,
    listen_addr: &str,
    jwks_cache_ttl: std::time::Duration,
    admin_org_id: Option<pigeon_domain::organization::OrganizationId>,
    mut shutdown: watch::Receiver<bool>,
    metrics_render: Arc<dyn Fn() -> String + Send + Sync>,
) -> anyhow::Result<()> {
    let uow_factory = Arc::new(PgUnitOfWorkFactory::new(pool.clone()));
    let read_store = Arc::new(PgApplicationReadStore::new(pool.clone()));
    let event_type_read_store = Arc::new(PgEventTypeReadStore::new(pool.clone()));
    let endpoint_read_store = Arc::new(PgEndpointReadStore::new(pool.clone()));
    let organization_read_store = Arc::new(PgOrganizationReadStore::new(pool.clone()));
    let oidc_config_read_store = Arc::new(PgOidcConfigReadStore::new(pool.clone()));
    let message_read_store = Arc::new(PgMessageReadStore::new(pool.clone()));
    let attempt_read_store = Arc::new(PgAttemptReadStore::new(pool.clone()));
    let dead_letter_read_store = Arc::new(PgDeadLetterReadStore::new(pool.clone()));
    let stats_read_store = Arc::new(PgStatsReadStore::new(pool.clone()));
    let event_type_stats_read_store = Arc::new(PgEventTypeStatsReadStore::new(pool.clone()));
    let endpoint_stats_read_store = Arc::new(PgEndpointStatsReadStore::new(pool.clone()));
    let health_checker = Arc::new(PgHealthChecker::new(pool.clone()));
    let audit_read_store = Arc::new(PgAuditReadStore::new(pool.clone()));
    let audit_store = Arc::new(PgAuditStore::new(pool));

    let idempotency_ttl = Duration::hours(24);

    let state = AppState {
        create_application: Arc::new(CreateApplicationHandler::new(uow_factory.clone())),
        update_application: Arc::new(UpdateApplicationHandler::new(uow_factory.clone())),
        delete_application: Arc::new(DeleteApplicationHandler::new(uow_factory.clone())),
        send_message: Arc::new(SendMessageHandler::new(
            uow_factory.clone(),
            endpoint_read_store.clone(),
            idempotency_ttl,
        )),
        get_application: Arc::new(GetApplicationByIdHandler::new(read_store.clone())),
        list_applications: Arc::new(ListApplicationsHandler::new(read_store.clone())),
        create_event_type: Arc::new(CreateEventTypeHandler::new(uow_factory.clone(), event_type_read_store.clone())),
        update_event_type: Arc::new(UpdateEventTypeHandler::new(uow_factory.clone())),
        delete_event_type: Arc::new(DeleteEventTypeHandler::new(uow_factory.clone())),
        get_event_type: Arc::new(GetEventTypeByIdHandler::new(event_type_read_store.clone())),
        list_event_types: Arc::new(ListEventTypesByAppHandler::new(event_type_read_store.clone())),
        create_endpoint: Arc::new(CreateEndpointHandler::new(uow_factory.clone(), event_type_read_store.clone())),
        update_endpoint: Arc::new(UpdateEndpointHandler::new(uow_factory.clone(), event_type_read_store.clone())),
        delete_endpoint: Arc::new(DeleteEndpointHandler::new(uow_factory.clone())),
        get_endpoint: Arc::new(GetEndpointByIdHandler::new(endpoint_read_store.clone())),
        list_endpoints: Arc::new(ListEndpointsByAppHandler::new(endpoint_read_store.clone())),
        create_organization: Arc::new(CreateOrganizationHandler::new(uow_factory.clone(), organization_read_store.clone())),
        update_organization: Arc::new(UpdateOrganizationHandler::new(uow_factory.clone())),
        delete_organization: Arc::new(DeleteOrganizationHandler::new(uow_factory.clone())),
        get_organization: Arc::new(GetOrganizationByIdHandler::new(
            organization_read_store.clone(),
        )),
        list_organizations: Arc::new(ListOrganizationsHandler::new(organization_read_store.clone())),
        create_oidc_config: Arc::new(CreateOidcConfigHandler::new(uow_factory.clone())),
        delete_oidc_config: Arc::new(DeleteOidcConfigHandler::new(uow_factory.clone())),
        get_oidc_config: Arc::new(GetOidcConfigByIdHandler::new(
            oidc_config_read_store.clone(),
        )),
        list_oidc_configs: Arc::new(ListOidcConfigsByOrgHandler::new(
            oidc_config_read_store.clone(),
        )),
        oidc_config_read_store,
        org_read_store: organization_read_store,
        jwks_provider: Arc::new(CachedJwksProvider::new(jwks_cache_ttl)),
        get_app_stats: Arc::new(GetAppStatsHandler::new(stats_read_store)),
        get_event_type_stats: Arc::new(GetEventTypeStatsHandler::new(event_type_stats_read_store)),
        get_endpoint_stats: Arc::new(GetEndpointStatsHandler::new(endpoint_stats_read_store)),
        get_message: Arc::new(GetMessageByIdHandler::new(message_read_store.clone())),
        list_messages: Arc::new(ListMessagesByAppHandler::new(message_read_store.clone())),
        list_attempts: Arc::new(ListAttemptsByMessageHandler::new(attempt_read_store.clone())),
        get_dead_letter: Arc::new(GetDeadLetterByIdHandler::new(dead_letter_read_store.clone())),
        list_dead_letters: Arc::new(ListDeadLettersByAppHandler::new(dead_letter_read_store)),
        replay_dead_letter: Arc::new(ReplayDeadLetterHandler::new(uow_factory.clone())),
        retry_attempt: Arc::new(RetryAttemptHandler::new(uow_factory.clone())),
        retrigger_message: Arc::new(RetriggerMessageHandler::new(uow_factory.clone(), message_read_store.clone(), endpoint_read_store, attempt_read_store.clone())),
        send_test_event: Arc::new(SendTestEventHandler::new(
            uow_factory.clone(),
            event_type_read_store.clone(),
        )),
        health_checker,
        list_audit_log: Arc::new(ListAuditLogHandler::new(audit_read_store)),
        audit_store,
        metrics_render,
        admin_org_id,
    };

    let router = pigeon_api::router(state);

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("Failed to bind to {}", listen_addr))?;

    info!("Listening on {}", listen_addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown.changed().await;
        })
        .await
        .context("Server error")?;

    Ok(())
}
