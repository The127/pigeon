use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::oidc_config::{OidcConfig, OidcConfigId};

use crate::error::ApplicationError;
use crate::mediator::handler::QueryHandler;
use crate::mediator::query::Query;
use crate::ports::stores::OidcConfigReadStore;

#[derive(Debug)]
pub struct GetOidcConfigById {
    pub id: OidcConfigId,
}

impl Query for GetOidcConfigById {
    type Output = Option<OidcConfig>;
}

pub struct GetOidcConfigByIdHandler {
    read_store: Arc<dyn OidcConfigReadStore>,
}

impl GetOidcConfigByIdHandler {
    pub fn new(read_store: Arc<dyn OidcConfigReadStore>) -> Self {
        Self { read_store }
    }
}

#[async_trait]
impl QueryHandler<GetOidcConfigById> for GetOidcConfigByIdHandler {
    async fn handle(
        &self,
        query: GetOidcConfigById,
    ) -> Result<Option<OidcConfig>, ApplicationError> {
        self.read_store.find_by_id(&query.id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::stores::MockOidcConfigReadStore;
    use pigeon_domain::oidc_config::OidcConfigState;

    #[tokio::test]
    async fn returns_config_when_found() {
        let config = OidcConfig::reconstitute(OidcConfigState::fake());
        let expected_issuer = config.issuer_url().to_string();
        let id = config.id().clone();
        let query_id = config.id().clone();

        let mut mock = MockOidcConfigReadStore::new();
        mock.expect_find_by_id()
            .withf(move |arg_id| *arg_id == id)
            .returning(move |_| Ok(Some(config.clone())));

        let handler = GetOidcConfigByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetOidcConfigById { id: query_id })
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().issuer_url(), expected_issuer);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let mut mock = MockOidcConfigReadStore::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let handler = GetOidcConfigByIdHandler::new(Arc::new(mock));
        let result = handler
            .handle(GetOidcConfigById {
                id: OidcConfigId::new(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
