use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::application::ApplicationResponse;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(as = PaginatedApplicationResponse)]
pub struct PaginatedResponse {
    pub items: Vec<ApplicationResponse>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}
