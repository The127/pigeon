use async_trait::async_trait;

use crate::error::ApplicationError;

use super::command::Command;
use super::query::Query;

#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<C::Output, ApplicationError>;
}

#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Output, ApplicationError>;
}
