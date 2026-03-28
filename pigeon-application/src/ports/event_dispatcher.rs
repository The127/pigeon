use async_trait::async_trait;

use pigeon_domain::event::DomainEvent;

use crate::error::ApplicationError;

/// An entry from the event outbox table, ready for processing.
#[derive(Debug, Clone)]
pub struct OutboxEntry {
    pub id: pigeon_domain::outbox::OutboxEntryId,
    pub event: DomainEvent,
}

/// Port for polling and processing the transactional event outbox.
#[cfg_attr(feature = "test-support", mockall::automock)]
#[async_trait]
pub trait EventOutbox: Send + Sync {
    /// Fetch unprocessed outbox entries, oldest first.
    async fn poll(&self, limit: u32) -> Result<Vec<OutboxEntry>, ApplicationError>;

    /// Mark an outbox entry as processed.
    async fn mark_processed(
        &self,
        id: &pigeon_domain::outbox::OutboxEntryId,
    ) -> Result<(), ApplicationError>;
}
