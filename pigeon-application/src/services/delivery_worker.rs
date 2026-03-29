use std::sync::Arc;

use chrono::Utc;
use metrics::{counter, gauge, histogram};
use tracing::{info, warn, Instrument};

use crate::ports::delivery::{DeliveryQueue, DeliveryTask, WebhookHttpClient, WebhookResult};

/// Configuration for the delivery worker.
#[derive(Debug, Clone)]
pub struct DeliveryWorkerConfig {
    pub batch_size: u32,
    pub poll_interval: std::time::Duration,
    pub max_retries: u32,
    pub backoff_base_secs: u64,
    pub max_backoff_secs: u64,
    /// How often to run the idempotency key cleanup job.
    pub cleanup_interval: std::time::Duration,
}

impl Default for DeliveryWorkerConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            poll_interval: std::time::Duration::from_millis(1000),
            max_retries: 5,
            backoff_base_secs: 30,
            max_backoff_secs: 3600,
            cleanup_interval: std::time::Duration::from_secs(3600),
        }
    }
}

pub struct DeliveryWorkerService {
    queue: Arc<dyn DeliveryQueue>,
    http_client: Arc<dyn WebhookHttpClient>,
    config: DeliveryWorkerConfig,
}

impl DeliveryWorkerService {
    pub fn new(
        queue: Arc<dyn DeliveryQueue>,
        http_client: Arc<dyn WebhookHttpClient>,
        config: DeliveryWorkerConfig,
    ) -> Self {
        Self {
            queue,
            http_client,
            config,
        }
    }

    /// Run the delivery worker loop until the shutdown signal is received.
    ///
    /// `shutdown`: send `true` to signal graceful stop. The worker finishes
    /// its current batch before exiting.
    pub async fn run(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        info!(
            batch_size = self.config.batch_size,
            poll_interval_ms = self.config.poll_interval.as_millis() as u64,
            max_retries = self.config.max_retries,
            "Delivery worker started"
        );

        let mut last_cleanup = tokio::time::Instant::now();

        loop {
            if *shutdown.borrow() {
                break;
            }

            // Periodic idempotency key cleanup
            if last_cleanup.elapsed() >= self.config.cleanup_interval {
                self.run_cleanup().await;
                last_cleanup = tokio::time::Instant::now();
            }

            match self.poll_and_deliver().await {
                Ok(0) => {
                    // No work — sleep before next poll
                    tokio::select! {
                        _ = tokio::time::sleep(self.config.poll_interval) => {}
                        _ = shutdown.changed() => {}
                    }
                }
                Ok(delivered) => {
                    info!(delivered, "Batch delivered");
                }
                Err(e) => {
                    warn!(error = %e, "Delivery poll error, will retry");
                    tokio::select! {
                        _ = tokio::time::sleep(self.config.poll_interval) => {}
                        _ = shutdown.changed() => {}
                    }
                }
            }
        }

        info!("Delivery worker stopped");
    }

    async fn run_cleanup(&self) {
        match self.queue.expire_idempotency_keys(Utc::now()).await {
            Ok(0) => {}
            Ok(expired) => {
                info!(expired, "Expired idempotency keys cleaned up");
            }
            Err(e) => {
                warn!(error = %e, "Failed to clean up idempotency keys");
            }
        }
    }

    /// Dequeue a batch and deliver each task. Returns the number of tasks processed.
    pub async fn poll_and_deliver(
        &self,
    ) -> Result<usize, crate::error::ApplicationError> {
        let tasks = self.queue.dequeue(self.config.batch_size).await?;
        let count = tasks.len();
        gauge!("pigeon_queue_depth").set(count as f64);

        for task in tasks {
            let span = tracing::info_span!(
                "deliver",
                attempt_id = ?task.attempt_id,
                message_id = ?task.message_id,
                endpoint_id = ?task.endpoint_id,
                attempt_number = task.attempt_number,
            );
            self.deliver_one(task).instrument(span).await;
        }

        Ok(count)
    }

    async fn deliver_one(&self, task: DeliveryTask) {
        let result = self
            .http_client
            .deliver(&task.endpoint_url, &task.payload, task.signing_secret.as_deref())
            .await;

        match result {
            WebhookResult::Response {
                status_code,
                body,
                duration_ms,
            } => {
                histogram!("pigeon_delivery_duration_seconds")
                    .record(duration_ms as f64 / 1000.0);

                if (200..300).contains(&status_code) {
                    counter!("pigeon_delivery_total", "status" => "success").increment(1);
                    if let Err(e) = self
                        .queue
                        .record_success(
                            &task.attempt_id,
                            &task.message_id,
                            &task.endpoint_id,
                            status_code,
                            body,
                            duration_ms,
                        )
                        .await
                    {
                        warn!(error = %e, "Failed to record success");
                    }
                } else {
                    self.handle_failure(
                        &task,
                        Some(status_code),
                        Some(body),
                        duration_ms,
                    )
                    .await;
                }
            }
            WebhookResult::Error {
                message,
                duration_ms,
            } => {
                histogram!("pigeon_delivery_duration_seconds")
                    .record(duration_ms as f64 / 1000.0);
                warn!(
                    attempt_id = ?task.attempt_id,
                    error = %message,
                    "Webhook delivery network error"
                );
                self.handle_failure(&task, None, None, duration_ms).await;
            }
        }
    }

    async fn handle_failure(
        &self,
        task: &DeliveryTask,
        response_code: Option<u16>,
        response_body: Option<String>,
        duration_ms: i64,
    ) {
        let retries_exhausted = task.attempt_number >= self.config.max_retries;

        let next_attempt_at = if retries_exhausted {
            None
        } else {
            Some(self.compute_next_attempt(task.attempt_number))
        };

        counter!("pigeon_delivery_total", "status" => "failure").increment(1);

        if let Err(e) = self
            .queue
            .record_failure(
                &task.attempt_id,
                &task.message_id,
                &task.endpoint_id,
                response_code,
                response_body.clone(),
                duration_ms,
                next_attempt_at,
            )
            .await
        {
            warn!(error = %e, "Failed to record failure");
            return;
        }

        if retries_exhausted {
            counter!("pigeon_delivery_total", "status" => "dead_letter").increment(1);
            info!(
                attempt_id = ?task.attempt_id,
                attempt_number = task.attempt_number,
                "Retries exhausted, dead lettering"
            );
            if let Err(e) = self
                .queue
                .insert_dead_letter(
                    &task.endpoint_id,
                    &task.message_id,
                    &task.app_id,
                    response_code,
                    response_body,
                )
                .await
            {
                warn!(error = %e, "Failed to insert dead letter");
            }
        }
    }

    fn compute_next_attempt(&self, current_attempt: u32) -> chrono::DateTime<Utc> {
        let delay_secs = self
            .config
            .backoff_base_secs
            .saturating_mul(2u64.saturating_pow(current_attempt - 1))
            .min(self.config.max_backoff_secs);

        Utc::now() + chrono::Duration::seconds(delay_secs as i64)
    }
}
