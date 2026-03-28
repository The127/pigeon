use async_trait::async_trait;
use pigeon_application::ports::health::HealthChecker;
use sqlx::PgPool;

pub struct PgHealthChecker {
    pool: PgPool,
}

impl PgHealthChecker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthChecker for PgHealthChecker {
    async fn check(&self) -> bool {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }
}
