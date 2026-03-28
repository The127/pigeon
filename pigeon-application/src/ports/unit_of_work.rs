use async_trait::async_trait;

use crate::error::ApplicationError;

use super::stores::{ApplicationStore, AttemptStore, DeadLetterStore, EndpointStore, EventTypeStore, MessageStore, OidcConfigStore, OrganizationStore};

// No #[automock] on UnitOfWork — commit/rollback take self: Box<Self> which is
// incompatible with mockall's generated mocks. Use FakeUnitOfWork from test_support
// for stateful transaction tracking instead.
#[async_trait]
pub trait UnitOfWork: Send {
    async fn commit(self: Box<Self>) -> Result<(), ApplicationError>;
    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError>;
    fn application_store(&mut self) -> &mut dyn ApplicationStore;
    fn event_type_store(&mut self) -> &mut dyn EventTypeStore;
    fn message_store(&mut self) -> &mut dyn MessageStore;
    fn attempt_store(&mut self) -> &mut dyn AttemptStore;
    fn dead_letter_store(&mut self) -> &mut dyn DeadLetterStore;
    fn endpoint_store(&mut self) -> &mut dyn EndpointStore;
    fn organization_store(&mut self) -> &mut dyn OrganizationStore;
    fn oidc_config_store(&mut self) -> &mut dyn OidcConfigStore;
}

// No #[automock] on UnitOfWorkFactory — returns Box<dyn UnitOfWork> which needs
// the hand-written FakeUnitOfWork. Use FakeUnitOfWorkFactory from test_support.
#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn UnitOfWork>, ApplicationError>;
}
