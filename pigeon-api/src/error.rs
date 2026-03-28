use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use pigeon_application::error::ApplicationError;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
    pub code: String,
}

pub struct ApiError(pub ApplicationError);

impl From<ApplicationError> for ApiError {
    fn from(err: ApplicationError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self.0 {
            ApplicationError::Validation(_) | ApplicationError::Domain(_) => {
                (StatusCode::BAD_REQUEST, "bad_request")
            }
            ApplicationError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            ApplicationError::Conflict => (StatusCode::CONFLICT, "conflict"),
            ApplicationError::UnitOfWork(_) | ApplicationError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };

        let body = ErrorBody {
            error: self.0.to_string(),
            code: code.to_string(),
        };

        (status, axum::Json(body)).into_response()
    }
}
