use chrono::{DateTime, Utc};
use uuid::Uuid;

use pigeon_macros::Reconstitute;

use crate::application::ApplicationId;
use crate::endpoint::EndpointId;
use crate::message::MessageId;
use crate::version::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadLetterId(Uuid);

impl Default for DeadLetterId {
    fn default() -> Self {
        Self::new()
    }
}

impl DeadLetterId {
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

#[derive(Debug, Clone, Reconstitute)]
pub struct DeadLetter {
    id: DeadLetterId,
    message_id: MessageId,
    endpoint_id: EndpointId,
    app_id: ApplicationId,
    last_response_code: Option<u16>,
    last_response_body: Option<String>,
    dead_lettered_at: DateTime<Utc>,
    replayed_at: Option<DateTime<Utc>>,
    version: Version,
}

impl DeadLetter {
    pub fn new(
        message_id: MessageId,
        endpoint_id: EndpointId,
        app_id: ApplicationId,
        last_response_code: Option<u16>,
        last_response_body: Option<String>,
    ) -> Self {
        Self {
            id: DeadLetterId::new(),
            message_id,
            endpoint_id,
            app_id,
            last_response_code,
            last_response_body,
            dead_lettered_at: Utc::now(),
            replayed_at: None,
            version: Version::new(0),
        }
    }

    pub fn mark_replayed(&mut self) {
        self.replayed_at = Some(Utc::now());
    }

    pub fn id(&self) -> &DeadLetterId {
        &self.id
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    pub fn endpoint_id(&self) -> &EndpointId {
        &self.endpoint_id
    }

    pub fn app_id(&self) -> &ApplicationId {
        &self.app_id
    }

    pub fn last_response_code(&self) -> Option<u16> {
        self.last_response_code
    }

    pub fn last_response_body(&self) -> Option<&str> {
        self.last_response_body.as_deref()
    }

    pub fn dead_lettered_at(&self) -> &DateTime<Utc> {
        &self.dead_lettered_at
    }

    pub fn replayed_at(&self) -> Option<&DateTime<Utc>> {
        self.replayed_at.as_ref()
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::any_dead_letter;

    #[test]
    fn create_dead_letter() {
        let dl = DeadLetter::new(
            MessageId::new(),
            EndpointId::new(),
            ApplicationId::new(),
            Some(500),
            Some("Internal Server Error".into()),
        );

        assert_eq!(dl.last_response_code(), Some(500));
        assert!(dl.replayed_at().is_none());
    }

    #[test]
    fn mark_replayed() {
        let mut dl = any_dead_letter();

        dl.mark_replayed();

        assert!(dl.replayed_at().is_some());
    }

    #[test]
    fn new_dead_letter_has_version_zero() {
        let dl = any_dead_letter();
        assert_eq!(dl.version(), Version::new(0));
    }

    #[test]
    fn new_dead_letter_has_no_replay() {
        let dl = DeadLetter::new(
            MessageId::new(),
            EndpointId::new(),
            ApplicationId::new(),
            Some(502),
            Some("Bad Gateway".into()),
        );

        assert!(dl.replayed_at().is_none());
        assert_eq!(dl.last_response_code(), Some(502));
        assert_eq!(dl.last_response_body(), Some("Bad Gateway"));
    }

    #[test]
    fn new_dead_letter_without_response() {
        let dl = DeadLetter::new(
            MessageId::new(),
            EndpointId::new(),
            ApplicationId::new(),
            None,
            None,
        );

        assert!(dl.last_response_code().is_none());
        assert!(dl.last_response_body().is_none());
    }

    #[test]
    fn dead_letter_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = DeadLetterId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn dead_letter_id_new_is_unique() {
        let id1 = DeadLetterId::new();
        let id2 = DeadLetterId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let state = DeadLetterState::fake();

        let dl = DeadLetter::reconstitute(DeadLetterState {
            id: state.id.clone(),
            message_id: state.message_id.clone(),
            endpoint_id: state.endpoint_id.clone(),
            app_id: state.app_id.clone(),
            last_response_code: state.last_response_code,
            last_response_body: state.last_response_body.clone(),
            dead_lettered_at: state.dead_lettered_at,
            replayed_at: state.replayed_at,
            version: state.version,
        });

        assert_eq!(*dl.id(), state.id);
        assert_eq!(*dl.message_id(), state.message_id);
        assert_eq!(*dl.endpoint_id(), state.endpoint_id);
        assert_eq!(*dl.app_id(), state.app_id);
        assert_eq!(dl.last_response_code(), state.last_response_code);
        assert_eq!(dl.last_response_body(), state.last_response_body.as_deref());
        assert_eq!(*dl.dead_lettered_at(), state.dead_lettered_at);
        assert_eq!(dl.replayed_at(), state.replayed_at.as_ref());
        assert_eq!(dl.version(), state.version);
    }
}
