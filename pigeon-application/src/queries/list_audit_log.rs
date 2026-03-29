use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::organization::OrganizationId;

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::audit_read_store::{AuditLogEntry, AuditReadStore};
use crate::queries::PaginatedResult;

#[derive(Debug)]
pub struct ListAuditLog {
    pub org_id: OrganizationId,
    pub command_filter: Option<String>,
    pub success_filter: Option<bool>,
    pub offset: u64,
    pub limit: u64,
}

impl Query for ListAuditLog {
    type Output = PaginatedResult<AuditLogEntry>;
}

pub struct ListAuditLogHandler {
    read_store: Arc<dyn AuditReadStore>,
}

impl ListAuditLogHandler {
    pub fn new(read_store: Arc<dyn AuditReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<ListAuditLog> for ListAuditLogHandler {
    async fn handle(
        &self,
        query: ListAuditLog,
    ) -> Result<PaginatedResult<AuditLogEntry>, ApplicationError> {
        let items = self.read_store.list_by_org(&query.org_id, query.command_filter.clone(), query.success_filter, query.offset, query.limit).await?;
        let total = self.read_store.count_by_org(&query.org_id, query.command_filter, query.success_filter).await?;
        Ok(PaginatedResult {
            items,
            total,
            offset: query.offset,
            limit: query.limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::audit_read_store::MockAuditReadStore;

    #[tokio::test]
    async fn returns_empty_list() {
        let mut mock = MockAuditReadStore::new();
        mock.expect_list_by_org().returning(|_, _, _, _, _| Ok(vec![]));
        mock.expect_count_by_org().returning(|_, _, _| Ok(0));

        let handler = ListAuditLogHandler::new(Arc::new(mock));
        let result = handler
            .handle(ListAuditLog {
                org_id: OrganizationId::new(),
                command_filter: None,
                success_filter: None,
                offset: 0,
                limit: 20,
            })
            .await
            .unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.total, 0);
    }
}
