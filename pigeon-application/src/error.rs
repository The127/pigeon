use pigeon_domain::error::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("unit of work error: {0}")]
    UnitOfWork(String),
    #[error("not found")]
    NotFound,
    #[error("conflict: resource was modified by another request")]
    Conflict,
    #[error("internal error: {0}")]
    Internal(String),
}
