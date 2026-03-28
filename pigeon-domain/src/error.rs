#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("conflict: resource was modified by another request")]
    Conflict,
    #[error("resource not found")]
    NotFound,
}
