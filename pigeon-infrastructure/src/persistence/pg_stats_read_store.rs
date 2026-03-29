use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stats_read_store::{AppStats, StatsReadStore, TimeBucket};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::organization::OrganizationId;

pub struct PgStatsReadStore {
    pool: PgPool,
}

impl PgStatsReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CountsRow {
    total_messages: i64,
    total_attempts: i64,
    total_succeeded: i64,
    total_failed: i64,
    total_dead_lettered: i64,
}

#[derive(sqlx::FromRow)]
struct BucketRow {
    bucket: DateTime<Utc>,
    succeeded: i64,
    failed: i64,
}

#[async_trait]
impl StatsReadStore for PgStatsReadStore {
    async fn get_app_stats(
        &self,
        app_id: &ApplicationId,
        org_id: &OrganizationId,
        since: DateTime<Utc>,
        bucket_interval_hours: u32,
    ) -> Result<AppStats, ApplicationError> {
        // Verify app belongs to org (SQL-level enforcement)
        let counts = sqlx::query_as::<_, CountsRow>(
            "SELECT \
                (SELECT COUNT(*) FROM messages m \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND a.org_id = $2 AND m.created_at >= $3) AS total_messages, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN messages m ON m.id = att.message_id \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND a.org_id = $2 AND att.attempted_at >= $3) AS total_attempts, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN messages m ON m.id = att.message_id \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND a.org_id = $2 AND att.status = 'succeeded' AND att.attempted_at >= $3) AS total_succeeded, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN messages m ON m.id = att.message_id \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND a.org_id = $2 AND att.status = 'failed' AND att.attempted_at >= $3) AS total_failed, \
                (SELECT COUNT(*) FROM dead_letters dl \
                 JOIN applications a ON a.id = dl.app_id \
                 WHERE dl.app_id = $1 AND a.org_id = $2 AND dl.dead_lettered_at >= $3) AS total_dead_lettered",
        )
        .bind(app_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        // Time-bucketed delivery results
        let buckets = sqlx::query_as::<_, BucketRow>(
            "SELECT \
                date_trunc('hour', att.attempted_at) \
                    - (EXTRACT(hour FROM att.attempted_at)::int % $4) * INTERVAL '1 hour' AS bucket, \
                COUNT(*) FILTER (WHERE att.status = 'succeeded') AS succeeded, \
                COUNT(*) FILTER (WHERE att.status = 'failed') AS failed \
             FROM attempts att \
             JOIN messages m ON m.id = att.message_id \
             JOIN applications a ON a.id = m.app_id \
             WHERE m.app_id = $1 AND a.org_id = $2 \
               AND att.attempted_at >= $3 \
               AND att.attempted_at IS NOT NULL \
               AND att.status IN ('succeeded', 'failed') \
             GROUP BY bucket \
             ORDER BY bucket ASC",
        )
        .bind(app_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(since)
        .bind(bucket_interval_hours as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let total_attempts = counts.total_attempts as u64;
        let total_succeeded = counts.total_succeeded as u64;
        let success_rate = if total_attempts > 0 {
            total_succeeded as f64 / total_attempts as f64
        } else {
            0.0
        };

        Ok(AppStats {
            total_messages: counts.total_messages as u64,
            total_attempts,
            total_succeeded,
            total_failed: counts.total_failed as u64,
            total_dead_lettered: counts.total_dead_lettered as u64,
            success_rate,
            time_series: buckets
                .into_iter()
                .map(|b| TimeBucket {
                    bucket: b.bucket,
                    succeeded: b.succeeded as u64,
                    failed: b.failed as u64,
                })
                .collect(),
        })
    }
}
