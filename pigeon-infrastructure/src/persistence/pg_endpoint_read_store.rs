use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::EndpointReadStore;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::endpoint::{Endpoint, EndpointId, EndpointState};
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

pub struct PgEndpointReadStore {

    pool: PgPool,
}

impl PgEndpointReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EndpointReadStore for PgEndpointReadStore {
    async fn find_enabled_by_app_and_event_type(
        &self,
        app_id: &ApplicationId,
        event_type_id: &EventTypeId,
        org_id: &OrganizationId,
    ) -> Result<Vec<Endpoint>, ApplicationError> {
        let rows = sqlx::query_as::<_, EndpointRow>(
            "SELECT e.id, e.app_id, e.url, e.signing_secret, e.enabled, e.created_at, \
             e.xmin::text::bigint AS version \
             FROM endpoints e \
             JOIN applications a ON a.id = e.app_id \
             JOIN endpoint_events ee ON ee.endpoint_id = e.id \
             WHERE e.app_id = $1 AND ee.event_type_id = $2 AND a.org_id = $3 AND e.enabled = true",
        )
        .bind(app_id.as_uuid())
        .bind(event_type_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let mut endpoints = Vec::with_capacity(rows.len());
        for row in rows {
            let event_type_ids = sqlx::query_scalar::<_, uuid::Uuid>(
                "SELECT event_type_id FROM endpoint_events WHERE endpoint_id = $1",
            )
            .bind(row.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

            endpoints.push(row.into_endpoint(
                event_type_ids
                    .into_iter()
                    .map(EventTypeId::from_uuid)
                    .collect(),
            ));
        }

        Ok(endpoints)
    }

    async fn find_by_id(
        &self,
        id: &EndpointId,
        org_id: &OrganizationId,
    ) -> Result<Option<Endpoint>, ApplicationError> {
        let row = sqlx::query_as::<_, EndpointRow>(
            "SELECT e.id, e.app_id, e.url, e.signing_secret, e.enabled, e.created_at, \
             e.xmin::text::bigint AS version \
             FROM endpoints e \
             JOIN applications a ON a.id = e.app_id \
             WHERE e.id = $1 AND a.org_id = $2",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        match row {
            Some(row) => {
                let event_type_ids = sqlx::query_scalar::<_, uuid::Uuid>(
                    "SELECT event_type_id FROM endpoint_events WHERE endpoint_id = $1",
                )
                .bind(row.id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| ApplicationError::Internal(e.to_string()))?;

                Ok(Some(row.into_endpoint(
                    event_type_ids
                        .into_iter()
                        .map(EventTypeId::from_uuid)
                        .collect(),
                )))
            }
            None => Ok(None),
        }
    }

    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Endpoint>, ApplicationError> {
        let rows = sqlx::query_as::<_, EndpointRow>(
            "SELECT e.id, e.app_id, e.url, e.signing_secret, e.enabled, e.created_at, \
             e.xmin::text::bigint AS version \
             FROM endpoints e \
             JOIN applications a ON a.id = e.app_id \
             WHERE e.app_id = $1 AND a.org_id = $2 \
             ORDER BY e.created_at DESC \
             LIMIT $3 OFFSET $4",
        )
        .bind(app_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let mut endpoints = Vec::with_capacity(rows.len());
        for row in rows {
            let event_type_ids = sqlx::query_scalar::<_, uuid::Uuid>(
                "SELECT event_type_id FROM endpoint_events WHERE endpoint_id = $1",
            )
            .bind(row.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

            endpoints.push(row.into_endpoint(
                event_type_ids
                    .into_iter()
                    .map(EventTypeId::from_uuid)
                    .collect(),
            ));
        }

        Ok(endpoints)
    }

    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
    ) -> Result<u64, ApplicationError> {
        let row: (i64,) =
            sqlx::query_as(
                "SELECT COUNT(*) FROM endpoints e \
                 JOIN applications a ON a.id = e.app_id \
                 WHERE e.app_id = $1 AND a.org_id = $2",
            )
                .bind(app_id.as_uuid())
                .bind(org_id.as_uuid())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }
}

#[derive(sqlx::FromRow)]
struct EndpointRow {
    id: uuid::Uuid,
    app_id: uuid::Uuid,
    url: String,
    signing_secret: String,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl EndpointRow {
    fn into_endpoint(self, event_type_ids: Vec<EventTypeId>) -> Endpoint {
        Endpoint::reconstitute(EndpointState {
            id: EndpointId::from_uuid(self.id),
            app_id: ApplicationId::from_uuid(self.app_id),
            url: self.url,
            signing_secret: self.signing_secret,
            enabled: self.enabled,
            event_type_ids,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}
