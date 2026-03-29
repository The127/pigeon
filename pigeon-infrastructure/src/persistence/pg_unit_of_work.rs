use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::{AttemptStore, DeadLetterStore, EndpointStore, EventTypeStore, MessageStore, OidcConfigStore, OrganizationStore};
use pigeon_application::ports::unit_of_work::UnitOfWork;
use sqlx::PgPool;

use super::change_tracker::{Change, ChangeTracker};
use super::pg_application_store::PgApplicationStore;
use super::pg_attempt_store::PgAttemptStore;
use super::pg_dead_letter_store::PgDeadLetterStore;
use super::pg_endpoint_store::PgEndpointStore;
use super::pg_event_type_store::PgEventTypeStore;
use super::pg_message_store::PgMessageStore;
use super::pg_oidc_config_store::PgOidcConfigStore;
use super::pg_organization_store::PgOrganizationStore;

pub(crate) struct PgUnitOfWork {
    pool: PgPool,
    tracker: Arc<Mutex<ChangeTracker>>,
    application_store: PgApplicationStore,
    event_type_store: PgEventTypeStore,
    endpoint_store: PgEndpointStore,
    message_store: PgMessageStore,
    attempt_store: PgAttemptStore,
    dead_letter_store: PgDeadLetterStore,
    organization_store: PgOrganizationStore,
    oidc_config_store: PgOidcConfigStore,
}

impl PgUnitOfWork {
    pub(crate) fn new(pool: PgPool) -> Self {
        let tracker = Arc::new(Mutex::new(ChangeTracker::new()));
        let application_store = PgApplicationStore::new(pool.clone(), Arc::clone(&tracker));
        let event_type_store = PgEventTypeStore::new(pool.clone(), Arc::clone(&tracker));
        let endpoint_store = PgEndpointStore::new(pool.clone(), Arc::clone(&tracker));
        let message_store = PgMessageStore::new(pool.clone(), Arc::clone(&tracker));
        let attempt_store = PgAttemptStore::new(pool.clone(), Arc::clone(&tracker));
        let dead_letter_store = PgDeadLetterStore::new(pool.clone(), Arc::clone(&tracker));
        let organization_store = PgOrganizationStore::new(pool.clone(), Arc::clone(&tracker));
        let oidc_config_store = PgOidcConfigStore::new(pool.clone(), Arc::clone(&tracker));
        Self {
            pool,
            tracker,
            application_store,
            event_type_store,
            endpoint_store,
            message_store,
            attempt_store,
            dead_letter_store,
            organization_store,
            oidc_config_store,
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    async fn commit(self: Box<Self>) -> Result<(), ApplicationError> {
        let (changes, events) = {
            let mut tracker = self.tracker.lock().unwrap();
            let events = tracker.collect_events();
            let changes = tracker.drain();
            (changes, events)
        };

        if changes.is_empty() && events.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

        for change in changes {
            match change {
                Change::ExplicitEvent(_) => {
                    // Handled by collect_events(), no SQL needed
                }
                Change::InsertApplication(app) => {
                    sqlx::query(
                        "INSERT INTO applications (id, org_id, name, uid, created_at) \
                         VALUES ($1, $2, $3, $4, $5)",
                    )
                    .bind(app.id().as_uuid())
                    .bind(app.org_id().as_uuid())
                    .bind(app.name())
                    .bind(app.uid())
                    .bind(app.created_at())
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::SaveApplication(app) => {
                    let result = sqlx::query(
                        "UPDATE applications SET name = $1, uid = $2 \
                         WHERE id = $3 AND xmin::text::bigint = $4",
                    )
                    .bind(app.name())
                    .bind(app.uid())
                    .bind(app.id().as_uuid())
                    .bind(app.version().as_u64() as i64)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    if result.rows_affected() == 0 {
                        return Err(ApplicationError::Conflict);
                    }
                }
                Change::DeleteApplication(id) => {
                    sqlx::query("DELETE FROM applications WHERE id = $1")
                        .bind(id.as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::InsertEventType(et) => {
                    sqlx::query(
                        "INSERT INTO event_types (id, app_id, name, schema, system, created_at) \
                         VALUES ($1, $2, $3, $4, $5, $6)",
                    )
                    .bind(et.id().as_uuid())
                    .bind(et.app_id().as_uuid())
                    .bind(et.name())
                    .bind(et.schema())
                    .bind(et.system())
                    .bind(et.created_at())
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::SaveEventType(et) => {
                    let result = sqlx::query(
                        "UPDATE event_types SET name = $1, schema = $2 \
                         WHERE id = $3 AND xmin::text::bigint = $4",
                    )
                    .bind(et.name())
                    .bind(et.schema())
                    .bind(et.id().as_uuid())
                    .bind(et.version().as_u64() as i64)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    if result.rows_affected() == 0 {
                        return Err(ApplicationError::Conflict);
                    }
                }
                Change::DeleteEventType(id) => {
                    sqlx::query("DELETE FROM event_types WHERE id = $1")
                        .bind(id.as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::InsertEndpoint(ep) => {
                    sqlx::query(
                        "INSERT INTO endpoints (id, app_id, url, signing_secret, enabled, created_at) \
                         VALUES ($1, $2, $3, $4, $5, $6)",
                    )
                    .bind(ep.id().as_uuid())
                    .bind(ep.app_id().as_uuid())
                    .bind(ep.url())
                    .bind(ep.signing_secret())
                    .bind(ep.enabled())
                    .bind(ep.created_at())
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    for et_id in ep.event_type_ids() {
                        sqlx::query(
                            "INSERT INTO endpoint_events (endpoint_id, event_type_id) \
                             VALUES ($1, $2)",
                        )
                        .bind(ep.id().as_uuid())
                        .bind(et_id.as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                    }
                }
                Change::SaveEndpoint(ep) => {
                    let result = sqlx::query(
                        "UPDATE endpoints SET url = $1, signing_secret = $2, enabled = $3 \
                         WHERE id = $4 AND xmin::text::bigint = $5",
                    )
                    .bind(ep.url())
                    .bind(ep.signing_secret())
                    .bind(ep.enabled())
                    .bind(ep.id().as_uuid())
                    .bind(ep.version().as_u64() as i64)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    if result.rows_affected() == 0 {
                        return Err(ApplicationError::Conflict);
                    }

                    // Replace endpoint_events
                    sqlx::query("DELETE FROM endpoint_events WHERE endpoint_id = $1")
                        .bind(ep.id().as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    for et_id in ep.event_type_ids() {
                        sqlx::query(
                            "INSERT INTO endpoint_events (endpoint_id, event_type_id) \
                             VALUES ($1, $2)",
                        )
                        .bind(ep.id().as_uuid())
                        .bind(et_id.as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                    }
                }
                Change::DeleteEndpoint(id) => {
                    sqlx::query("DELETE FROM endpoint_events WHERE endpoint_id = $1")
                        .bind(id.as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    sqlx::query("DELETE FROM endpoints WHERE id = $1")
                        .bind(id.as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::InsertMessage(msg) => {
                    sqlx::query(
                        "INSERT INTO messages \
                         (id, app_id, event_type_id, payload, idempotency_key, \
                          idempotency_expires_at, created_at) \
                         VALUES ($1, $2, $3, $4, $5, $6, $7) \
                         ON CONFLICT (app_id, idempotency_key) \
                         DO NOTHING",
                    )
                    .bind(msg.id().as_uuid())
                    .bind(msg.app_id().as_uuid())
                    .bind(msg.event_type_id().as_uuid())
                    .bind(msg.payload())
                    .bind(msg.idempotency_key().as_str())
                    .bind(msg.idempotency_expires_at())
                    .bind(msg.created_at())
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::InsertOrganization(org) => {
                    sqlx::query(
                        "INSERT INTO organizations (id, name, slug, created_at) \
                         VALUES ($1, $2, $3, $4)",
                    )
                    .bind(org.id().as_uuid())
                    .bind(org.name())
                    .bind(org.slug())
                    .bind(org.created_at())
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::SaveOrganization(org) => {
                    let result = sqlx::query(
                        "UPDATE organizations SET name = $1 \
                         WHERE id = $2 AND xmin::text::bigint = $3",
                    )
                    .bind(org.name())
                    .bind(org.id().as_uuid())
                    .bind(org.version().as_u64() as i64)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    if result.rows_affected() == 0 {
                        return Err(ApplicationError::Conflict);
                    }
                }
                Change::DeleteOrganization(id) => {
                    sqlx::query("DELETE FROM organizations WHERE id = $1")
                        .bind(id.as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::InsertOidcConfig(config) => {
                    sqlx::query(
                        "INSERT INTO oidc_configs (id, org_id, issuer_url, audience, jwks_url, created_at) \
                         VALUES ($1, $2, $3, $4, $5, $6)",
                    )
                    .bind(config.id().as_uuid())
                    .bind(config.org_id().as_uuid())
                    .bind(config.issuer_url())
                    .bind(config.audience())
                    .bind(config.jwks_url())
                    .bind(config.created_at())
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::DeleteOidcConfig(id) => {
                    sqlx::query("DELETE FROM oidc_configs WHERE id = $1")
                        .bind(id.as_uuid())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::InsertAttempt(attempt) => {
                    let status_str = match attempt.status() {
                        pigeon_domain::attempt::AttemptStatus::Pending => "pending",
                        pigeon_domain::attempt::AttemptStatus::InFlight => "in_flight",
                        pigeon_domain::attempt::AttemptStatus::Succeeded => "succeeded",
                        pigeon_domain::attempt::AttemptStatus::Failed => "failed",
                    };
                    sqlx::query(
                        "INSERT INTO attempts \
                         (id, message_id, endpoint_id, status, next_attempt_at, attempt_number, created_at) \
                         VALUES ($1, $2, $3, $4, $5, $6, now())",
                    )
                    .bind(attempt.id().as_uuid())
                    .bind(attempt.message_id().as_uuid())
                    .bind(attempt.endpoint_id().as_uuid())
                    .bind(status_str)
                    .bind(attempt.next_attempt_at())
                    .bind(attempt.attempt_number() as i32)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::SaveAttempt(attempt) => {
                    let status_str = match attempt.status() {
                        pigeon_domain::attempt::AttemptStatus::Pending => "pending",
                        pigeon_domain::attempt::AttemptStatus::InFlight => "in_flight",
                        pigeon_domain::attempt::AttemptStatus::Succeeded => "succeeded",
                        pigeon_domain::attempt::AttemptStatus::Failed => "failed",
                    };
                    let rows = sqlx::query(
                        "UPDATE attempts \
                         SET status = $2, next_attempt_at = $3 \
                         WHERE id = $1 AND xmin::text::bigint = $4",
                    )
                    .bind(attempt.id().as_uuid())
                    .bind(status_str)
                    .bind(attempt.next_attempt_at())
                    .bind(attempt.version().as_u64() as i64)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    if rows.rows_affected() == 0 {
                        return Err(ApplicationError::Conflict);
                    }
                }
                Change::InsertDeadLetter(dl) => {
                    sqlx::query(
                        "INSERT INTO dead_letters \
                         (id, message_id, endpoint_id, app_id, last_response_code, \
                          last_response_body, dead_lettered_at) \
                         VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    )
                    .bind(dl.id().as_uuid())
                    .bind(dl.message_id().as_uuid())
                    .bind(dl.endpoint_id().as_uuid())
                    .bind(dl.app_id().as_uuid())
                    .bind(dl.last_response_code().map(|c| c as i16))
                    .bind(dl.last_response_body())
                    .bind(dl.dead_lettered_at())
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
                }
                Change::SaveDeadLetter(dl) => {
                    let rows = sqlx::query(
                        "UPDATE dead_letters SET replayed_at = $2 \
                         WHERE id = $1 AND xmin::text::bigint = $3",
                    )
                    .bind(dl.id().as_uuid())
                    .bind(dl.replayed_at())
                    .bind(dl.version().as_u64() as i64)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

                    if rows.rows_affected() == 0 {
                        return Err(ApplicationError::Conflict);
                    }
                }
            }
        }

        // Insert domain events into outbox (same transaction)
        for event in &events {
            sqlx::query(
                "INSERT INTO event_outbox (id, event_type, payload) \
                 VALUES ($1, $2, $3)",
            )
            .bind(uuid::Uuid::new_v4())
            .bind(event.event_type())
            .bind(event.to_json())
            .execute(&mut *tx)
            .await
            .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| ApplicationError::UnitOfWork(e.to_string()))?;

        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), ApplicationError> {
        // No DB transaction was opened — just discard in-memory changes
        Ok(())
    }

    fn application_store(&mut self) -> &mut dyn pigeon_application::ports::stores::ApplicationStore {
        &mut self.application_store
    }

    fn event_type_store(&mut self) -> &mut dyn EventTypeStore {
        &mut self.event_type_store
    }

    fn message_store(&mut self) -> &mut dyn MessageStore {
        &mut self.message_store
    }

    fn attempt_store(&mut self) -> &mut dyn AttemptStore {
        &mut self.attempt_store
    }

    fn dead_letter_store(&mut self) -> &mut dyn DeadLetterStore {
        &mut self.dead_letter_store
    }

    fn endpoint_store(&mut self) -> &mut dyn EndpointStore {
        &mut self.endpoint_store
    }

    fn organization_store(&mut self) -> &mut dyn OrganizationStore {
        &mut self.organization_store
    }

    fn oidc_config_store(&mut self) -> &mut dyn OidcConfigStore {
        &mut self.oidc_config_store
    }

    fn emit_event(&mut self, event: pigeon_domain::event::DomainEvent) {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::ExplicitEvent(event));
    }
}
