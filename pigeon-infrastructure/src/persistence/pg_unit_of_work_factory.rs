use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};
use sqlx::PgPool;

use super::pg_unit_of_work::PgUnitOfWork;

pub struct PgUnitOfWorkFactory {
    pool: PgPool,
}

impl PgUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWorkFactory for PgUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn UnitOfWork>, ApplicationError> {
        Ok(Box::new(PgUnitOfWork::new(self.pool.clone())))
    }
}
