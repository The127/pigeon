use std::sync::Arc;

use async_trait::async_trait;
use tracing::warn;

use crate::ports::projection_store::ProjectionStore;
use crate::services::outbox_worker::EventSubscriber;
use pigeon_domain::event::DomainEvent;

/// Outbox subscriber that maintains denormalized read model projections
/// for endpoint delivery summaries and message delivery status.
pub struct DeliveryProjectionSubscriber {
    store: Arc<dyn ProjectionStore>,
}

impl DeliveryProjectionSubscriber {
    pub fn new(store: Arc<dyn ProjectionStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl EventSubscriber for DeliveryProjectionSubscriber {
    async fn handle(&self, event: &DomainEvent) {
        let result = match event {
            DomainEvent::MessageCreated {
                message_id,
                attempts_created,
                ..
            } => {
                self.store
                    .init_message_status(message_id, *attempts_created)
                    .await
            }
            DomainEvent::AttemptSucceeded {
                endpoint_id,
                message_id,
                ..
            } => {
                let r1 = self
                    .store
                    .record_endpoint_success(endpoint_id, chrono::Utc::now())
                    .await;
                let r2 = self.store.increment_message_succeeded(message_id).await;
                r1.and(r2)
            }
            DomainEvent::AttemptFailed {
                endpoint_id,
                message_id,
                will_retry,
                ..
            } => {
                let r1 = self
                    .store
                    .record_endpoint_failure(endpoint_id, chrono::Utc::now())
                    .await;
                if !will_retry {
                    let r2 = self.store.increment_message_failed(message_id).await;
                    r1.and(r2)
                } else {
                    r1
                }
            }
            DomainEvent::DeadLettered { message_id, .. } => {
                self.store
                    .increment_message_dead_lettered(message_id)
                    .await
            }
            _ => Ok(()),
        };

        if let Err(e) = result {
            warn!(error = %e, event_type = event.event_type(), "Projection update failed");
        }
    }
}
