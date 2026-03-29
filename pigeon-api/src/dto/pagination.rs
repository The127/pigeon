use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::application::ApplicationResponse;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ApplicationListQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct MessageListQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub event_type_id: Option<uuid::Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct DeadLetterListQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub endpoint_id: Option<uuid::Uuid>,
    pub replayed: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct AuditLogListQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub command: Option<String>,
    pub success: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(as = PaginatedApplicationResponse)]
pub struct PaginatedResponse {
    pub items: Vec<ApplicationResponse>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}
