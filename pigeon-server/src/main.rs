use std::sync::Arc;

use anyhow::Context;
use chrono::Duration;
use clap::{Parser, Subcommand};
use sqlx::PgPool;
use tokio::sync::watch;
use tracing::info;

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
use pigeon_application::commands::send_message::SendMessageHandler;
use pigeon_application::commands::update_application::UpdateApplicationHandler;
use pigeon_application::commands::update_endpoint::UpdateEndpointHandler;
use pigeon_application::commands::update_event_type::UpdateEventTypeHandler;
use pigeon_application::commands::update_organization::UpdateOrganizationHandler;
use pigeon_application::queries::get_application_by_id::GetApplicationByIdHandler;
use pigeon_application::queries::get_endpoint_by_id::GetEndpointByIdHandler;
use pigeon_application::queries::get_event_type_by_id::GetEventTypeByIdHandler;
use pigeon_application::queries::get_oidc_config_by_id::GetOidcConfigByIdHandler;
use pigeon_application::queries::get_organization_by_id::GetOrganizationByIdHandler;
use pigeon_application::queries::list_applications::ListApplicationsHandler;
use pigeon_application::queries::list_endpoints_by_app::ListEndpointsByAppHandler;
use pigeon_application::queries::list_event_types_by_app::ListEventTypesByAppHandler;
use pigeon_application::queries::list_oidc_configs_by_org::ListOidcConfigsByOrgHandler;
use pigeon_application::queries::list_organizations::ListOrganizationsHandler;
use pigeon_application::services::delivery_worker::{DeliveryWorkerConfig, DeliveryWorkerService};
use pigeon_infrastructure::http::ReqwestWebhookClient;
use pigeon_infrastructure::persistence::{
    PgApplicationReadStore, PgDeliveryQueue, PgEndpointReadStore, PgEventTypeReadStore,
    PgHealthChecker, PgOidcConfigReadStore, PgOrganizationReadStore, PgUnitOfWorkFactory,
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

    match cli.command {
        Commands::Serve => {
            let pool = create_pool(&config).await?;
            run_migrations(&pool).await?;
            run_bootstrap(&pool, &config).await?;

            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            let api_pool = pool.clone();
            let api_config = config.listen_addr.clone();
            let jwks_ttl = config.jwks_cache_ttl;
            let api_shutdown_rx = shutdown_rx.clone();

            let api_handle = tokio::spawn(async move {
                run_api(api_pool, &api_config, jwks_ttl, api_shutdown_rx).await
            });

            let worker = create_worker(&pool, &config);
            let worker_handle = tokio::spawn(async move { worker.run(shutdown_rx).await });

            tokio::signal::ctrl_c()
                .await
                .context("Failed to listen for ctrl-c")?;
            info!("Shutdown signal received");
            let _ = shutdown_tx.send(true);

            let (api_result, _) = tokio::join!(api_handle, worker_handle);
            api_result??;
        }
        Commands::Api => {
            let pool = create_pool(&config).await?;
            run_bootstrap(&pool, &config).await?;

            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            let api_handle = tokio::spawn(async move {
                run_api(pool, &config.listen_addr, config.jwks_cache_ttl, shutdown_rx).await
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
            let worker_handle = tokio::spawn(async move { worker.run(shutdown_rx).await });

            tokio::signal::ctrl_c()
                .await
                .context("Failed to listen for ctrl-c")?;
            info!("Shutdown signal received");
            let _ = shutdown_tx.send(true);

            worker_handle.await?;
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
    };

    DeliveryWorkerService::new(queue, http_client, worker_config)
}

async fn run_api(
    pool: PgPool,
    listen_addr: &str,
    jwks_cache_ttl: std::time::Duration,
    mut shutdown: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let uow_factory = Arc::new(PgUnitOfWorkFactory::new(pool.clone()));
    let read_store = Arc::new(PgApplicationReadStore::new(pool.clone()));
    let event_type_read_store = Arc::new(PgEventTypeReadStore::new(pool.clone()));
    let endpoint_read_store = Arc::new(PgEndpointReadStore::new(pool.clone()));
    let organization_read_store = Arc::new(PgOrganizationReadStore::new(pool.clone()));
    let oidc_config_read_store = Arc::new(PgOidcConfigReadStore::new(pool.clone()));
    let health_checker = Arc::new(PgHealthChecker::new(pool));

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
        create_event_type: Arc::new(CreateEventTypeHandler::new(uow_factory.clone())),
        update_event_type: Arc::new(UpdateEventTypeHandler::new(uow_factory.clone())),
        delete_event_type: Arc::new(DeleteEventTypeHandler::new(uow_factory.clone())),
        get_event_type: Arc::new(GetEventTypeByIdHandler::new(event_type_read_store.clone())),
        list_event_types: Arc::new(ListEventTypesByAppHandler::new(event_type_read_store)),
        create_endpoint: Arc::new(CreateEndpointHandler::new(uow_factory.clone())),
        update_endpoint: Arc::new(UpdateEndpointHandler::new(uow_factory.clone())),
        delete_endpoint: Arc::new(DeleteEndpointHandler::new(uow_factory.clone())),
        get_endpoint: Arc::new(GetEndpointByIdHandler::new(endpoint_read_store.clone())),
        list_endpoints: Arc::new(ListEndpointsByAppHandler::new(endpoint_read_store)),
        create_organization: Arc::new(CreateOrganizationHandler::new(uow_factory.clone())),
        update_organization: Arc::new(UpdateOrganizationHandler::new(uow_factory.clone())),
        delete_organization: Arc::new(DeleteOrganizationHandler::new(uow_factory.clone())),
        get_organization: Arc::new(GetOrganizationByIdHandler::new(
            organization_read_store.clone(),
        )),
        list_organizations: Arc::new(ListOrganizationsHandler::new(organization_read_store)),
        create_oidc_config: Arc::new(CreateOidcConfigHandler::new(uow_factory.clone())),
        delete_oidc_config: Arc::new(DeleteOidcConfigHandler::new(uow_factory)),
        get_oidc_config: Arc::new(GetOidcConfigByIdHandler::new(
            oidc_config_read_store.clone(),
        )),
        list_oidc_configs: Arc::new(ListOidcConfigsByOrgHandler::new(
            oidc_config_read_store.clone(),
        )),
        oidc_config_read_store,
        app_read_store: read_store.clone(),
        jwks_provider: Arc::new(CachedJwksProvider::new(jwks_cache_ttl)),
        health_checker,
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
