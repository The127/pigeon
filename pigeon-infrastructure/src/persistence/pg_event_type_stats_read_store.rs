use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use pigeon_application::error::ApplicationError;
use pigeon_application::ports::event_type_stats_read_store::{
    EventTypeStats, EventTypeStatsReadStore, EventTypeTimeBucket, RecentMessage,
};
use pigeon_domain::application::ApplicationId;
use pigeon_domain::event_type::EventTypeId;
use pigeon_domain::organization::OrganizationId;

pub struct PgEventTypeStatsReadStore {
    pool: PgPool,
}

impl PgEventTypeStatsReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CountsRow {
    total_messages: i64,
    total_attempts: i64,
    total_pending: i64,
    total_succeeded: i64,
    total_failed: i64,
    total_dead_lettered: i64,
    subscribed_endpoints: i64,
}

#[derive(sqlx::FromRow)]
struct BucketRow {
    bucket: DateTime<Utc>,
    succeeded: i64,
    failed: i64,
}

#[derive(sqlx::FromRow)]
struct RecentMessageRow {
    id: uuid::Uuid,
    idempotency_key: String,
    created_at: DateTime<Utc>,
    attempts_created: Option<i32>,
    succeeded: Option<i32>,
    failed: Option<i32>,
    dead_lettered: Option<i32>,
}

#[async_trait]
impl EventTypeStatsReadStore for PgEventTypeStatsReadStore {
    async fn get_stats(
        &self,
        app_id: &ApplicationId,
        event_type_id: &EventTypeId,
        org_id: &OrganizationId,
        since: DateTime<Utc>,
        bucket_interval_hours: u32,
    ) -> Result<EventTypeStats, ApplicationError> {
        let counts = sqlx::query_as::<_, CountsRow>(
            "SELECT \
                (SELECT COUNT(*) FROM messages m \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND m.event_type_id = $2 AND a.org_id = $3 \
                   AND m.created_at >= $4) AS total_messages, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN messages m ON m.id = att.message_id \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND m.event_type_id = $2 AND a.org_id = $3 \
                   AND att.attempted_at >= $4) AS total_attempts, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN messages m ON m.id = att.message_id \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND m.event_type_id = $2 AND a.org_id = $3 \
                   AND att.status IN ('pending', 'in_flight')) AS total_pending, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN messages m ON m.id = att.message_id \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND m.event_type_id = $2 AND a.org_id = $3 \
                   AND att.status = 'succeeded' AND att.attempted_at >= $4) AS total_succeeded, \
                (SELECT COUNT(*) FROM attempts att \
                 JOIN messages m ON m.id = att.message_id \
                 JOIN applications a ON a.id = m.app_id \
                 WHERE m.app_id = $1 AND m.event_type_id = $2 AND a.org_id = $3 \
                   AND att.status = 'failed' AND att.attempted_at >= $4) AS total_failed, \
                (SELECT COUNT(*) FROM dead_letters dl \
                 JOIN messages m ON m.id = dl.message_id \
                 JOIN applications a ON a.id = dl.app_id \
                 WHERE dl.app_id = $1 AND m.event_type_id = $2 AND a.org_id = $3 \
                   AND dl.dead_lettered_at >= $4) AS total_dead_lettered, \
                (SELECT COUNT(*) FROM endpoint_events ee \
                 JOIN endpoints e ON e.id = ee.endpoint_id \
                 JOIN applications a ON a.id = e.app_id \
                 WHERE ee.event_type_id = $2 AND e.app_id = $1 AND a.org_id = $3 AND e.enabled = true) AS subscribed_endpoints",
        )
        .bind(app_id.as_uuid())
        .bind(event_type_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let buckets = sqlx::query_as::<_, BucketRow>(
            "SELECT \
                date_trunc('hour', att.attempted_at) \
                    - (EXTRACT(hour FROM att.attempted_at)::int % $5) * INTERVAL '1 hour' AS bucket, \
                COUNT(*) FILTER (WHERE att.status = 'succeeded') AS succeeded, \
                COUNT(*) FILTER (WHERE att.status = 'failed') AS failed \
             FROM attempts att \
             JOIN messages m ON m.id = att.message_id \
             JOIN applications a ON a.id = m.app_id \
             WHERE m.app_id = $1 AND m.event_type_id = $2 AND a.org_id = $3 \
               AND att.attempted_at >= $4 \
               AND att.attempted_at IS NOT NULL \
               AND att.status IN ('succeeded', 'failed') \
             GROUP BY bucket \
             ORDER BY bucket ASC",
        )
        .bind(app_id.as_uuid())
        .bind(event_type_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(since)
        .bind(bucket_interval_hours as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        let recent = sqlx::query_as::<_, RecentMessageRow>(
            "SELECT m.id, m.idempotency_key, m.created_at, \
                    mds.attempts_created, mds.succeeded, mds.failed, mds.dead_lettered \
             FROM messages m \
             JOIN applications a ON a.id = m.app_id \
             LEFT JOIN message_delivery_status mds ON mds.message_id = m.id \
             WHERE m.app_id = $1 AND m.event_type_id = $2 AND a.org_id = $3 \
             ORDER BY m.created_at DESC \
             LIMIT 10",
        )
        .bind(app_id.as_uuid())
        .bind(event_type_id.as_uuid())
        .bind(org_id.as_uuid())
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

        Ok(EventTypeStats {
            total_messages: counts.total_messages as u64,
            total_attempts,
            total_pending: counts.total_pending as u64,
            total_succeeded,
            total_failed: counts.total_failed as u64,
            total_dead_lettered: counts.total_dead_lettered as u64,
            success_rate,
            subscribed_endpoints: counts.subscribed_endpoints as u64,
            time_series: buckets
                .into_iter()
                .map(|b| EventTypeTimeBucket {
                    bucket: b.bucket,
                    succeeded: b.succeeded as u64,
                    failed: b.failed as u64,
                })
                .collect(),
            recent_messages: recent
                .into_iter()
                .map(|r| RecentMessage {
                    id: r.id,
                    idempotency_key: r.idempotency_key,
                    created_at: r.created_at,
                    attempts_created: r.attempts_created.unwrap_or(0) as u32,
                    succeeded: r.succeeded.unwrap_or(0) as u32,
                    failed: r.failed.unwrap_or(0) as u32,
                    dead_lettered: r.dead_lettered.unwrap_or(0) as u32,
                })
                .collect(),
        })
    }
}
