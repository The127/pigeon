use chrono::{DateTime, Utc};
use uuid::Uuid;

use pigeon_macros::Reconstitute;

use crate::endpoint::EndpointId;
use crate::message::MessageId;
use crate::version::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptId(Uuid);

impl Default for AttemptId {
    fn default() -> Self {
        Self::new()
    }
}

impl AttemptId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptStatus {
    Pending,
    InFlight,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Reconstitute)]
pub struct Attempt {
    id: AttemptId,
    message_id: MessageId,
    endpoint_id: EndpointId,
    status: AttemptStatus,
    response_code: Option<u16>,
    response_body: Option<String>,
    attempted_at: Option<DateTime<Utc>>,
    next_attempt_at: Option<DateTime<Utc>>,
    attempt_number: u32,
    duration_ms: Option<i64>,
    version: Version,
}

impl Attempt {
    pub fn new(
        message_id: MessageId,
        endpoint_id: EndpointId,
        next_attempt_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: AttemptId::new(),
            message_id,
            endpoint_id,
            status: AttemptStatus::Pending,
            response_code: None,
            response_body: None,
            attempted_at: None,
            next_attempt_at: Some(next_attempt_at),
            attempt_number: 1,
            duration_ms: None,
            version: Version::new(0),
        }
    }

    pub fn mark_in_flight(&mut self) {
        self.status = AttemptStatus::InFlight;
    }

    pub fn mark_for_retry(&mut self, next_attempt_at: DateTime<Utc>) {
        self.status = AttemptStatus::Pending;
        self.next_attempt_at = Some(next_attempt_at);
    }

    pub fn record_success(
        &mut self,
        response_code: u16,
        response_body: String,
        duration_ms: i64,
    ) {
        self.status = AttemptStatus::Succeeded;
        self.response_code = Some(response_code);
        self.response_body = Some(response_body);
        self.attempted_at = Some(Utc::now());
        self.next_attempt_at = None;
        self.duration_ms = Some(duration_ms);
    }

    pub fn record_failure(
        &mut self,
        response_code: Option<u16>,
        response_body: Option<String>,
        duration_ms: Option<i64>,
        next_attempt_at: Option<DateTime<Utc>>,
    ) {
        self.status = if next_attempt_at.is_some() {
            AttemptStatus::Pending
        } else {
            AttemptStatus::Failed
        };
        self.response_code = response_code;
        self.response_body = response_body;
        self.attempted_at = Some(Utc::now());
        self.next_attempt_at = next_attempt_at;
        self.duration_ms = duration_ms;
    }

    pub fn id(&self) -> &AttemptId {
        &self.id
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    pub fn endpoint_id(&self) -> &EndpointId {
        &self.endpoint_id
    }

    pub fn status(&self) -> AttemptStatus {
        self.status
    }

    pub fn response_code(&self) -> Option<u16> {
        self.response_code
    }

    pub fn response_body(&self) -> Option<&str> {
        self.response_body.as_deref()
    }

    pub fn attempted_at(&self) -> Option<&DateTime<Utc>> {
        self.attempted_at.as_ref()
    }

    pub fn next_attempt_at(&self) -> Option<&DateTime<Utc>> {
        self.next_attempt_at.as_ref()
    }

    pub fn attempt_number(&self) -> u32 {
        self.attempt_number
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.duration_ms
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::any_attempt;

    fn pending_attempt() -> Attempt {
        any_attempt()
    }

    #[test]
    fn new_attempt_is_pending() {
        let attempt = pending_attempt();

        assert_eq!(attempt.status(), AttemptStatus::Pending);
        assert!(attempt.attempted_at().is_none());
        assert!(attempt.next_attempt_at().is_some());
        assert_eq!(attempt.attempt_number(), 1);
        assert!(attempt.duration_ms().is_none());
    }

    #[test]
    fn mark_in_flight() {
        let mut attempt = pending_attempt();

        attempt.mark_in_flight();

        assert_eq!(attempt.status(), AttemptStatus::InFlight);
    }

    #[test]
    fn record_success() {
        let mut attempt = pending_attempt();

        attempt.record_success(200, "OK".into(), 150);

        assert_eq!(attempt.status(), AttemptStatus::Succeeded);
        assert_eq!(attempt.response_code(), Some(200));
        assert_eq!(attempt.response_body(), Some("OK"));
        assert!(attempt.attempted_at().is_some());
        assert!(attempt.next_attempt_at().is_none());
        assert_eq!(attempt.duration_ms(), Some(150));
    }

    #[test]
    fn record_failure_with_retry_goes_back_to_pending() {
        let mut attempt = pending_attempt();
        let next = Utc::now();

        attempt.record_failure(Some(500), Some("Internal Server Error".into()), Some(42), Some(next));

        assert_eq!(attempt.status(), AttemptStatus::Pending);
        assert_eq!(attempt.response_code(), Some(500));
        assert!(attempt.next_attempt_at().is_some());
        assert_eq!(attempt.duration_ms(), Some(42));
    }

    #[test]
    fn record_failure_without_retry_is_final() {
        let mut attempt = pending_attempt();

        attempt.record_failure(None, None, Some(10), None);

        assert_eq!(attempt.status(), AttemptStatus::Failed);
        assert!(attempt.next_attempt_at().is_none());
    }

    #[test]
    fn new_attempt_has_version_zero() {
        let attempt = pending_attempt();
        assert_eq!(attempt.version(), Version::new(0));
    }

    #[test]
    fn new_attempt_has_no_response() {
        let attempt = pending_attempt();
        assert!(attempt.response_code().is_none());
        assert!(attempt.response_body().is_none());
    }

    #[test]
    fn attempt_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = AttemptId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn attempt_id_new_is_unique() {
        let id1 = AttemptId::new();
        let id2 = AttemptId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let state = AttemptState::fake();

        let attempt = Attempt::reconstitute(AttemptState {
            id: state.id.clone(),
            message_id: state.message_id.clone(),
            endpoint_id: state.endpoint_id.clone(),
            status: state.status,
            response_code: state.response_code,
            response_body: state.response_body.clone(),
            attempted_at: state.attempted_at,
            next_attempt_at: state.next_attempt_at,
            attempt_number: state.attempt_number,
            duration_ms: state.duration_ms,
            version: state.version,
        });

        assert_eq!(*attempt.id(), state.id);
        assert_eq!(*attempt.message_id(), state.message_id);
        assert_eq!(*attempt.endpoint_id(), state.endpoint_id);
        assert_eq!(attempt.status(), state.status);
        assert_eq!(attempt.response_code(), state.response_code);
        assert_eq!(attempt.response_body(), state.response_body.as_deref());
        assert_eq!(attempt.attempted_at(), state.attempted_at.as_ref());
        assert_eq!(attempt.next_attempt_at(), state.next_attempt_at.as_ref());
        assert_eq!(attempt.attempt_number(), state.attempt_number);
        assert_eq!(attempt.duration_ms(), state.duration_ms);
        assert_eq!(attempt.version(), state.version);
    }
}
