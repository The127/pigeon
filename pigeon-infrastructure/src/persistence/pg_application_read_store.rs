use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::ApplicationReadStore;
use pigeon_domain::application::{Application, ApplicationId, ApplicationState};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;
use sqlx::PgPool;

pub struct PgApplicationReadStore {
    pool: PgPool,
}

impl PgApplicationReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApplicationReadStore for PgApplicationReadStore {
    async fn find_by_id(
        &self,
        id: &ApplicationId,
        org_id: &OrganizationId,
    ) -> Result<Option<Application>, ApplicationError> {
        let row = sqlx::query_as::<_, ApplicationRow>(
            "SELECT id, org_id, name, uid, created_at, xmin::text::bigint AS version \
             FROM applications WHERE id = $1 AND org_id = $2",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into_application()))
    }

    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
        search: Option<String>,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Application>, ApplicationError> {
        let mut sql = String::from(
            "SELECT id, org_id, name, uid, created_at, xmin::text::bigint AS version \
             FROM applications \
             WHERE org_id = $1",
        );
        let mut param_idx = 2u32;
        if search.is_some() {
            sql.push_str(&format!(
                " AND (name ILIKE '%' || ${p} || '%' OR uid ILIKE '%' || ${p} || '%')",
                p = param_idx,
            ));
            param_idx += 1;
        }
        sql.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1,
        ));

        let mut q = sqlx::query_as::<_, ApplicationRow>(&sql).bind(org_id.as_uuid());
        if let Some(s) = &search {
            q = q.bind(s.as_str());
        }
        let rows = q
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_application()).collect())
    }

    async fn count_by_org(
        &self,
        org_id: &OrganizationId,
        search: Option<String>,
    ) -> Result<u64, ApplicationError> {
        let mut sql = String::from("SELECT COUNT(*) FROM applications WHERE org_id = $1");
        if search.is_some() {
            sql.push_str(" AND (name ILIKE '%' || $2 || '%' OR uid ILIKE '%' || $2 || '%')");
        }

        let mut q = sqlx::query_as::<_, (i64,)>(&sql).bind(org_id.as_uuid());
        if let Some(s) = &search {
            q = q.bind(s.as_str());
        }
        let row = q
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        Ok(row.0 as u64)
    }
}

#[derive(sqlx::FromRow)]
struct ApplicationRow {
    id: uuid::Uuid,
    org_id: uuid::Uuid,
    name: String,
    uid: String,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl ApplicationRow {
    fn into_application(self) -> Application {
        Application::reconstitute(ApplicationState {
            id: ApplicationId::from_uuid(self.id),
            org_id: OrganizationId::from_uuid(self.org_id),
            name: self.name,
            uid: self.uid,
            created_at: self.created_at,
            version: Version::new(self.version as u64),
        })
    }
}
