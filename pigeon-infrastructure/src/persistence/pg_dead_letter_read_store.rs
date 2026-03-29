use async_trait::async_trait;
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::DeadLetterReadStore;
use pigeon_domain::application::ApplicationId;
use pigeon_domain::dead_letter::{DeadLetter, DeadLetterId, DeadLetterState};
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::message::MessageId;
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;

pub struct PgDeadLetterReadStore {
    pool: PgPool,
}

impl PgDeadLetterReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct DeadLetterRow {
    id: uuid::Uuid,
    message_id: uuid::Uuid,
    endpoint_id: uuid::Uuid,
    app_id: uuid::Uuid,
    last_response_code: Option<i16>,
    last_response_body: Option<String>,
    dead_lettered_at: chrono::DateTime<chrono::Utc>,
    replayed_at: Option<chrono::DateTime<chrono::Utc>>,
    version: i64,
}

impl DeadLetterRow {
    fn into_dead_letter(self) -> DeadLetter {
        DeadLetter::reconstitute(DeadLetterState {
            id: DeadLetterId::from_uuid(self.id),
            message_id: MessageId::from_uuid(self.message_id),
            endpoint_id: EndpointId::from_uuid(self.endpoint_id),
            app_id: ApplicationId::from_uuid(self.app_id),
            last_response_code: self.last_response_code.map(|c| c as u16),
            last_response_body: self.last_response_body,
            dead_lettered_at: self.dead_lettered_at,
            replayed_at: self.replayed_at,
            version: Version::new(self.version as u64),
        })
    }
}

#[async_trait]
impl DeadLetterReadStore for PgDeadLetterReadStore {
    async fn consecutive_failure_count(
        &self,
        endpoint_id: &EndpointId,
    ) -> Result<u64, ApplicationError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM dead_letters \
             WHERE endpoint_id = $1 \
               AND dead_lettered_at > COALESCE( \
                   (SELECT MAX(attempted_at) FROM attempts \
                    WHERE endpoint_id = $1 AND status = 'succeeded'), \
                   '1970-01-01'::timestamptz \
               )",
        )
        .bind(endpoint_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }

    async fn find_by_id(
        &self,
        id: &DeadLetterId,
        org_id: &OrganizationId,
    ) -> Result<Option<DeadLetter>, ApplicationError> {
        let row = sqlx::query_as::<_, DeadLetterRow>(
            "SELECT dl.id, dl.message_id, dl.endpoint_id, dl.app_id, \
                    dl.last_response_code, dl.last_response_body, \
                    dl.dead_lettered_at, dl.replayed_at, \
                    dl.xmin::text::bigint AS version \
             FROM dead_letters dl \
             JOIN applications a ON a.id = dl.app_id \
             WHERE dl.id = $1 AND a.org_id = $2",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_dead_letter()))
    }

    async fn list_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        endpoint_id: Option<EndpointId>,
        replayed: Option<bool>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<DeadLetter>, ApplicationError> {
        let mut sql = String::from(
            "SELECT dl.id, dl.message_id, dl.endpoint_id, dl.app_id, \
                    dl.last_response_code, dl.last_response_body, \
                    dl.dead_lettered_at, dl.replayed_at, \
                    dl.xmin::text::bigint AS version \
             FROM dead_letters dl \
             JOIN applications a ON a.id = dl.app_id \
             WHERE dl.app_id = $1 AND a.org_id = $2",
        );
        let mut param_idx = 3u32;
        if endpoint_id.is_some() {
            sql.push_str(&format!(" AND dl.endpoint_id = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(r) = replayed {
            if r {
                sql.push_str(" AND dl.replayed_at IS NOT NULL");
            } else {
                sql.push_str(" AND dl.replayed_at IS NULL");
            }
        }
        sql.push_str(&format!(
            " ORDER BY dl.dead_lettered_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1,
        ));

        let mut q = sqlx::query_as::<_, DeadLetterRow>(&sql)
            .bind(app_id.as_uuid())
            .bind(org_id.as_uuid());
        if let Some(ref ep_id) = endpoint_id {
            q = q.bind(ep_id.as_uuid());
        }
        let rows = q
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_dead_letter()).collect())
    }

    async fn count_by_app(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        endpoint_id: Option<EndpointId>,
        replayed: Option<bool>,
    ) -> Result<u64, ApplicationError> {
        let mut sql = String::from(
            "SELECT COUNT(*) FROM dead_letters dl \
             JOIN applications a ON a.id = dl.app_id \
             WHERE dl.app_id = $1 AND a.org_id = $2",
        );
        let mut param_idx = 3u32;
        if endpoint_id.is_some() {
            sql.push_str(&format!(" AND dl.endpoint_id = ${param_idx}"));
            param_idx += 1;
        }
        let _ = param_idx; // suppress unused warning
        if let Some(r) = replayed {
            if r {
                sql.push_str(" AND dl.replayed_at IS NOT NULL");
            } else {
                sql.push_str(" AND dl.replayed_at IS NULL");
            }
        }

        let mut q = sqlx::query_as::<_, (i64,)>(&sql)
            .bind(app_id.as_uuid())
            .bind(org_id.as_uuid());
        if let Some(ref ep_id) = endpoint_id {
            q = q.bind(ep_id.as_uuid());
        }
        let row = q
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }
}
