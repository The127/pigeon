use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use uuid::Uuid;

use pigeon_macros::Reconstitute;

use crate::application::ApplicationId;
use crate::event_type::EventTypeId;
use crate::version::Version;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(Uuid);

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageId {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(key: String) -> Self {
        Self(key)
    }

    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Reconstitute)]
pub struct Message {
    id: MessageId,
    app_id: ApplicationId,
    event_type_id: EventTypeId,
    payload: Value,
    idempotency_key: IdempotencyKey,
    idempotency_expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    version: Version,
}

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("payload must be a JSON object")]
    PayloadNotObject,
}

impl Message {
    pub fn new(
        app_id: ApplicationId,
        event_type_id: EventTypeId,
        payload: Value,
        idempotency_key: Option<String>,
        idempotency_ttl: Duration,
    ) -> Result<Self, MessageError> {
        if !payload.is_object() {
            return Err(MessageError::PayloadNotObject);
        }

        let now = Utc::now();
        let key = match idempotency_key {
            Some(k) => IdempotencyKey::new(k),
            None => IdempotencyKey::generate(),
        };

        Ok(Self {
            id: MessageId::new(),
            app_id,
            event_type_id,
            payload,
            idempotency_key: key,
            idempotency_expires_at: now + idempotency_ttl,
            created_at: now,
            version: Version::new(0),
        })
    }

    pub fn id(&self) -> &MessageId {
        &self.id
    }

    pub fn app_id(&self) -> &ApplicationId {
        &self.app_id
    }

    pub fn event_type_id(&self) -> &EventTypeId {
        &self.event_type_id
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }

    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    pub fn idempotency_expires_at(&self) -> &DateTime<Utc> {
        &self.idempotency_expires_at
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::test_support::any_message;

    fn default_ttl() -> Duration {
        Duration::hours(24)
    }

    #[test]
    fn create_message_with_object_payload() {
        let msg = Message::new(
            ApplicationId::new(),
            EventTypeId::new(),
            json!({"user_id": "u_123"}),
            Some("my-key".into()),
            default_ttl(),
        )
        .unwrap();

        assert!(msg.payload().is_object());
        assert_eq!(msg.idempotency_key().as_str(), "my-key");
    }

    #[test]
    fn generates_idempotency_key_when_none_supplied() {
        let msg = Message::new(
            ApplicationId::new(),
            EventTypeId::new(),
            json!({"user_id": "u_123"}),
            None,
            default_ttl(),
        )
        .unwrap();

        assert!(!msg.idempotency_key().as_str().is_empty());
    }

    #[test]
    fn idempotency_expires_at_is_in_the_future() {
        let msg = Message::new(
            ApplicationId::new(),
            EventTypeId::new(),
            json!({"data": true}),
            None,
            default_ttl(),
        )
        .unwrap();

        assert!(*msg.idempotency_expires_at() > *msg.created_at());
    }

    #[test]
    fn reject_non_object_payload() {
        let result = Message::new(
            ApplicationId::new(),
            EventTypeId::new(),
            json!("not an object"),
            None,
            default_ttl(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn reject_array_payload() {
        let result = Message::new(
            ApplicationId::new(),
            EventTypeId::new(),
            json!([1, 2, 3]),
            None,
            default_ttl(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn new_message_has_version_zero() {
        let msg = any_message();
        assert_eq!(msg.version(), Version::new(0));
    }

    #[test]
    fn idempotency_key_new_preserves_value() {
        let key = IdempotencyKey::new("my-key-123".into());
        assert_eq!(key.as_str(), "my-key-123");
    }

    #[test]
    fn idempotency_key_generate_produces_unique_values() {
        let k1 = IdempotencyKey::generate();
        let k2 = IdempotencyKey::generate();
        assert_ne!(k1, k2);
    }

    #[test]
    fn message_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = MessageId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn message_id_new_is_unique() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn message_id_default_equals_new() {
        let id = MessageId::default();
        // just verify it produces a valid id without panicking
        assert!(!id.as_uuid().is_nil());
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let state = MessageState::fake();

        let msg = Message::reconstitute(MessageState {
            id: state.id.clone(),
            app_id: state.app_id.clone(),
            event_type_id: state.event_type_id.clone(),
            payload: state.payload.clone(),
            idempotency_key: state.idempotency_key.clone(),
            idempotency_expires_at: state.idempotency_expires_at,
            created_at: state.created_at,
            version: state.version,
        });

        assert_eq!(*msg.id(), state.id);
        assert_eq!(*msg.app_id(), state.app_id);
        assert_eq!(*msg.event_type_id(), state.event_type_id);
        assert_eq!(*msg.payload(), state.payload);
        assert_eq!(*msg.idempotency_key(), state.idempotency_key);
        assert_eq!(*msg.idempotency_expires_at(), state.idempotency_expires_at);
        assert_eq!(*msg.created_at(), state.created_at);
        assert_eq!(msg.version(), state.version);
    }
}
