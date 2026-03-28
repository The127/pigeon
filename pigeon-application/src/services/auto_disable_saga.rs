use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, warn};

use crate::commands::disable_endpoint::DisableEndpoint;
use crate::mediator::handler::CommandHandler;
use crate::ports::stores::DeadLetterReadStore;
use crate::services::outbox_worker::EventSubscriber;
use pigeon_domain::event::DomainEvent;

/// Saga that auto-disables endpoints after N consecutive dead letters.
///
/// Subscribes to `DeadLettered` events. On each, queries the consecutive
/// failure count via `DeadLetterReadStore`. If it meets or exceeds the
/// threshold, sends a `DisableEndpoint` command through the command handler
/// (which goes through the full mediator pipeline when wired).
pub struct AutoDisableEndpointSaga {
    dead_letter_read_store: Arc<dyn DeadLetterReadStore>,
    disable_handler: Arc<dyn CommandHandler<DisableEndpoint>>,
    threshold: u64,
}

impl AutoDisableEndpointSaga {
    pub fn new(
        dead_letter_read_store: Arc<dyn DeadLetterReadStore>,
        disable_handler: Arc<dyn CommandHandler<DisableEndpoint>>,
        threshold: u64,
    ) -> Self {
        Self {
            dead_letter_read_store,
            disable_handler,
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

        if let Err(e) = self.disable_handler.handle(command).await {
            warn!(error = %e, "Failed to auto-disable endpoint");
        }
    }
}
