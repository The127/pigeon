use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use pigeon_macros::Reconstitute;

use crate::application::ApplicationId;
use crate::version::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTypeId(Uuid);

impl Default for EventTypeId {
    fn default() -> Self {
        Self::new()
    }
}

impl EventTypeId {
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
pub struct EventType {
    id: EventTypeId,
    app_id: ApplicationId,
    name: String,
    schema: Option<Value>,
    created_at: DateTime<Utc>,
    version: Version,
}

#[derive(Debug, thiserror::Error)]
pub enum EventTypeError {
    #[error("event type name must not be empty")]
    EmptyName,
}

impl EventType {
    pub fn new(
        app_id: ApplicationId,
        name: String,
        schema: Option<Value>,
    ) -> Result<Self, EventTypeError> {
        if name.trim().is_empty() {
            return Err(EventTypeError::EmptyName);
        }

        Ok(Self {
            id: EventTypeId::new(),
            app_id,
            name,
            schema,
            created_at: Utc::now(),
            version: Version::new(0),
        })
    }

    pub fn id(&self) -> &EventTypeId {
        &self.id
    }

    pub fn app_id(&self) -> &ApplicationId {
        &self.app_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema(&self) -> Option<&Value> {
        self.schema.as_ref()
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn update(&mut self, name: String, schema: Option<Value>) -> Result<(), EventTypeError> {
        if name.trim().is_empty() {
            return Err(EventTypeError::EmptyName);
        }
        self.name = name;
        self.schema = schema;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::test_support::any_event_type;

    #[test]
    fn create_event_type_without_schema() {
        let et = EventType::new(ApplicationId::new(), "user.created".into(), None).unwrap();

        assert_eq!(et.name(), "user.created");
        assert!(et.schema().is_none());
    }

    #[test]
    fn create_event_type_with_schema() {
        let schema = json!({"type": "object"});
        let et =
            EventType::new(ApplicationId::new(), "user.created".into(), Some(schema)).unwrap();

        assert!(et.schema().is_some());
    }

    #[test]
    fn reject_empty_name() {
        let result = EventType::new(ApplicationId::new(), "".into(), None);

        assert!(result.is_err());
    }

    #[test]
    fn reject_whitespace_only_name() {
        let result = EventType::new(ApplicationId::new(), "  ".into(), None);

        assert!(result.is_err());
    }

    #[test]
    fn new_event_type_has_version_zero() {
        let et = any_event_type();
        assert_eq!(et.version(), Version::new(0));
    }

    #[test]
    fn event_type_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = EventTypeId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn event_type_id_new_is_unique() {
        let id1 = EventTypeId::new();
        let id2 = EventTypeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let state = EventTypeState::fake();

        let et = EventType::reconstitute(EventTypeState {
            id: state.id.clone(),
            app_id: state.app_id.clone(),
            name: state.name.clone(),
            schema: state.schema.clone(),
            created_at: state.created_at,
            version: state.version,
        });

        assert_eq!(*et.id(), state.id);
        assert_eq!(*et.app_id(), state.app_id);
        assert_eq!(et.name(), state.name);
        assert_eq!(et.schema(), state.schema.as_ref());
        assert_eq!(*et.created_at(), state.created_at);
        assert_eq!(et.version(), state.version);
    }
}
