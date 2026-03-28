use async_trait::async_trait;

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check(&self) -> bool;
}
