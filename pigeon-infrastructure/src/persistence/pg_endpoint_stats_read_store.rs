use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::endpoint_stats_read_store::{
    EndpointStats, EndpointStatsReadStore, EndpointTimeBucket,
};
use pigeon_domain::endpoint::EndpointId;
use pigeon_domain::organization::OrganizationId;

pub struct PgEndpointStatsReadStore {
    pool: PgPool,
}

impl PgEndpointStatsReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CountsRow {
    total_attempts: i64,
    total_pending: i64,
    total_succeeded: i64,
    total_failed: i64,
    total_dead_lettered: i64,
    consecutive_failures: i64,
    last_delivery_at: Option<DateTime<Utc>>,
    last_status: Option<String>,
}

#[derive(sqlx::FromRow)]
struct BucketRow {
    bucket: DateTime<Utc>,
    succeeded: i64,
    failed: i64,
}

#[async_trait]
impl EndpointStatsReadStore for PgEndpointStatsReadStore {
    async fn get_stats(
        &self,
        endpoint_id: &EndpointId,
        org_id: &OrganizationId,
        since: DateTime<Utc>,
        bucket_interval_hours: u32,
    ) -> Result<EndpointStats, ApplicationError> {
        let counts = sqlx::query_as::<_, CountsRow>(
            "SELECT \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN endpoints e ON e.id = att.endpoint_id \
                 JOIN applications a ON a.id = e.app_id \
                 WHERE att.endpoint_id = $1 AND a.org_id = $2 \
                   AND att.attempted_at >= $3) AS total_attempts, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN endpoints e ON e.id = att.endpoint_id \
                 JOIN applications a ON a.id = e.app_id \
                 WHERE att.endpoint_id = $1 AND a.org_id = $2 \
                   AND att.status IN ('pending', 'in_flight')) AS total_pending, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN endpoints e ON e.id = att.endpoint_id \
                 JOIN applications a ON a.id = e.app_id \
                 WHERE att.endpoint_id = $1 AND a.org_id = $2 \
                   AND att.status = 'succeeded' AND att.attempted_at >= $3) AS total_succeeded, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN endpoints e ON e.id = att.endpoint_id \
                 JOIN applications a ON a.id = e.app_id \
                 WHERE att.endpoint_id = $1 AND a.org_id = $2 \
                   AND att.status = 'failed' AND att.attempted_at >= $3) AS total_failed, \
                (SELECT COUNT(*) FROM dead_letters dl \
                 JOIN endpoints e ON e.id = dl.endpoint_id \
                 JOIN applications a ON a.id = e.app_id \
                 WHERE dl.endpoint_id = $1 AND a.org_id = $2 \
                   AND dl.dead_lettered_at >= $3) AS total_dead_lettered, \
                COALESCE((SELECT eds.consecutive_failures FROM endpoint_delivery_summary eds \
                 WHERE eds.endpoint_id = $1), 0) AS consecutive_failures, \
                (SELECT eds.last_delivery_at FROM endpoint_delivery_summary eds \
                 WHERE eds.endpoint_id = $1) AS last_delivery_at, \
                (SELECT eds.last_status FROM endpoint_delivery_summary eds \
                 WHERE eds.endpoint_id = $1) AS last_status",
        )
        .bind(endpoint_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let buckets = sqlx::query_as::<_, BucketRow>(
            "SELECT \
                date_trunc('hour', att.attempted_at) \
                    - (EXTRACT(hour FROM att.attempted_at)::int % $4) * INTERVAL '1 hour' AS bucket, \
                COUNT(*) FILTER (WHERE att.status = 'succeeded') AS succeeded, \
                COUNT(*) FILTER (WHERE att.status = 'failed') AS failed \
             FROM attempts att \
             JOIN endpoints e ON e.id = att.endpoint_id \
             JOIN applications a ON a.id = e.app_id \
             WHERE att.endpoint_id = $1 AND a.org_id = $2 \
               AND att.attempted_at >= $3 \
               AND att.attempted_at IS NOT NULL \
               AND att.status IN ('succeeded', 'failed') \
             GROUP BY bucket \
             ORDER BY bucket ASC",
        )
        .bind(endpoint_id.as_uuid())
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

        Ok(EndpointStats {
            total_attempts,
            total_pending: counts.total_pending as u64,
            total_succeeded,
            total_failed: counts.total_failed as u64,
            total_dead_lettered: counts.total_dead_lettered as u64,
            success_rate,
            consecutive_failures: counts.consecutive_failures as u64,
            last_delivery_at: counts.last_delivery_at,
            last_status: counts.last_status,
            time_series: buckets
                .into_iter()
                .map(|b| EndpointTimeBucket {
                    bucket: b.bucket,
                    succeeded: b.succeeded as u64,
                    failed: b.failed as u64,
                })
                .collect(),
        })
    }
}
