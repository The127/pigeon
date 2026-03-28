use std::sync::Arc;

use tracing::{info, warn};

use crate::ports::event_dispatcher::EventOutbox;
use pigeon_domain::event::DomainEvent;

/// Configuration for the outbox worker.
#[derive(Debug, Clone)]
pub struct OutboxWorkerConfig {
    pub poll_interval: std::time::Duration,
    pub batch_size: u32,
}

impl Default for OutboxWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: std::time::Duration::from_millis(1000),
            batch_size: 50,
        }
    }
}

pub struct OutboxWorkerService {
    outbox: Arc<dyn EventOutbox>,
    config: OutboxWorkerConfig,
}

impl OutboxWorkerService {
    pub fn new(outbox: Arc<dyn EventOutbox>, config: OutboxWorkerConfig) -> Self {
        Self { outbox, config }
    }

    pub async fn run(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        info!("Outbox worker started");

        loop {
            if *shutdown.borrow() {
                break;
            }

            match self.poll_and_process().await {
                Ok(0) => {
                    tokio::select! {
                        _ = tokio::time::sleep(self.config.poll_interval) => {}
                        _ = shutdown.changed() => {}
                    }
                }
                Ok(processed) => {
                    info!(processed, "Outbox entries processed");
                }
                Err(e) => {
                    warn!(error = %e, "Outbox poll error, will retry");
                    tokio::select! {
                        _ = tokio::time::sleep(self.config.poll_interval) => {}
                        _ = shutdown.changed() => {}
                    }
                }
            }
        }

        info!("Outbox worker stopped");
    }

    async fn poll_and_process(&self) -> Result<usize, crate::error::ApplicationError> {
        let entries = self.outbox.poll(self.config.batch_size).await?;
        let count = entries.len();

        for entry in entries {
            self.handle_event(&entry.event);
            self.outbox.mark_processed(&entry.id).await?;
        }

        Ok(count)
    }

    fn handle_event(&self, event: &DomainEvent) {
        match event {
            DomainEvent::DeadLettered {
                message_id,
                endpoint_id,
                app_id,
            } => {
                info!(
                    message_id = ?message_id,
                    endpoint_id = ?endpoint_id,
                    app_id = ?app_id,
                    "Domain event: message dead-lettered"
                );
            }
        }
    }
}
