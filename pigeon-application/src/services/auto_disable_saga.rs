use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, warn};

use crate::commands::disable_endpoint::DisableEndpoint;
use crate::mediator::handler::CommandHandler;
use crate::mediator::pipeline::RequestContext;
use crate::ports::stores::DeadLetterReadStore;
use crate::ports::unit_of_work::UnitOfWorkFactory;
use crate::services::outbox_worker::EventSubscriber;
use pigeon_domain::event::DomainEvent;
use pigeon_domain::organization::OrganizationId;

/// Saga that auto-disables endpoints after N consecutive dead letters.
///
/// Subscribes to `DeadLettered` events. On each, queries the consecutive
/// failure count via `DeadLetterReadStore`. If it meets or exceeds the
/// threshold, sends a `DisableEndpoint` command through the command handler
/// (which goes through the full mediator pipeline when wired).
pub struct AutoDisableEndpointSaga {
    dead_letter_read_store: Arc<dyn DeadLetterReadStore>,
    disable_handler: Arc<dyn CommandHandler<DisableEndpoint>>,
    uow_factory: Arc<dyn UnitOfWorkFactory>,
    threshold: u64,
}

impl AutoDisableEndpointSaga {
    pub fn new(
        dead_letter_read_store: Arc<dyn DeadLetterReadStore>,
        disable_handler: Arc<dyn CommandHandler<DisableEndpoint>>,
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        threshold: u64,
    ) -> Self {
        Self {
            dead_letter_read_store,
            disable_handler,
            uow_factory,
            threshold,
        }
    }
}

#[async_trait]
impl EventSubscriber for AutoDisableEndpointSaga {
    async fn handle(&self, event: &DomainEvent) {
        let (endpoint_id, app_id) = match event {
            DomainEvent::DeadLettered {
                endpoint_id,
                app_id,
                ..
            } => (endpoint_id, app_id),
            _ => return,
        };

        if self.threshold == 0 {
            return;
        }

        let count = match self
            .dead_letter_read_store
            .consecutive_failure_count(endpoint_id)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, "Failed to check consecutive failure count");
                return;
            }
        };

        if count < self.threshold {
            return;
        }

        info!(
            endpoint_id = ?endpoint_id,
            consecutive_failures = count,
            threshold = self.threshold,
            "Auto-disabling endpoint due to consecutive failures"
        );

        let command = DisableEndpoint {
            app_id: app_id.clone(),
            endpoint_id: endpoint_id.clone(),
        };

        let uow = match self.uow_factory.begin().await {
            Ok(uow) => uow,
            Err(e) => {
                warn!(error = %e, "Failed to begin UoW for auto-disable");
                return;
            }
        };

        let mut ctx = RequestContext::new(
            "DisableEndpoint",
            "system:auto-disable-saga".into(),
            OrganizationId::new(), // internal saga, no tenant context
        );
        ctx.set_uow(uow);

        match self.disable_handler.handle(command, &mut ctx).await {
            Ok(()) => {
                if let Some(uow) = ctx.take_uow() {
                    if let Err(e) = uow.commit().await {
                        warn!(error = %e, "Failed to commit auto-disable");
                    }
                }
            }
            Err(e) => {
                if let Some(uow) = ctx.take_uow() {
                    let _ = uow.rollback().await;
                }
                warn!(error = %e, "Failed to auto-disable endpoint");
            }
        }
    }
}
