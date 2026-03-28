use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pigeon_application::error::ApplicationError;
use pigeon_application::ports::stores::AttemptStore;
use pigeon_domain::attempt::Attempt;

use super::change_tracker::{Change, ChangeTracker};

pub(crate) struct PgAttemptStore {
    tracker: Arc<Mutex<ChangeTracker>>,
}

impl PgAttemptStore {
    pub(crate) fn new(tracker: Arc<Mutex<ChangeTracker>>) -> Self {
        Self { tracker }
    }
}

#[async_trait]
impl AttemptStore for PgAttemptStore {
    async fn insert(&mut self, attempt: &Attempt) -> Result<(), ApplicationError> {
        self.tracker
            .lock()
            .unwrap()
            .record(Change::InsertAttempt(attempt.clone()));
        Ok(())
    }
}
