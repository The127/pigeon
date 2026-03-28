use chrono::{DateTime, Utc};
use pigeon_macros::Reconstitute;
use uuid::Uuid;

use crate::organization::OrganizationId;
use crate::version::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationId(Uuid);

impl Default for ApplicationId {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationId {
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
pub struct Application {
    id: ApplicationId,
    org_id: OrganizationId,
    name: String,
    uid: String,
    created_at: DateTime<Utc>,
    version: Version,
}

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("application name must not be empty")]
    EmptyName,
}

impl Application {
    pub fn new(org_id: OrganizationId, name: String, uid: String) -> Result<Self, ApplicationError> {
        if name.trim().is_empty() {
            return Err(ApplicationError::EmptyName);
        }

        Ok(Self {
            id: ApplicationId::new(),
            org_id,
            name,
            uid,
            created_at: Utc::now(),
            version: Version::new(0),
        })
    }

    pub fn id(&self) -> &ApplicationId {
        &self.id
    }

    pub fn org_id(&self) -> &OrganizationId {
        &self.org_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn rename(&mut self, name: String) -> Result<(), ApplicationError> {
        if name.trim().is_empty() {
            return Err(ApplicationError::EmptyName);
        }
        self.name = name;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organization::OrganizationId;
    use crate::test_support::any_application;

    #[test]
    fn create_application_with_valid_name() {
        let org_id = OrganizationId::new();
        let app = Application::new(org_id.clone(), "my-app".into(), "app_123".into()).unwrap();

        assert_eq!(app.name(), "my-app");
        assert_eq!(app.uid(), "app_123");
        assert_eq!(app.org_id(), &org_id);
    }

    #[test]
    fn reject_empty_name() {
        let result = Application::new(OrganizationId::new(), "".into(), "app_123".into());

        assert!(result.is_err());
    }

    #[test]
    fn reject_whitespace_only_name() {
        let result = Application::new(OrganizationId::new(), "   ".into(), "app_123".into());

        assert!(result.is_err());
    }

    #[test]
    fn new_application_has_version_zero() {
        let app = any_application();
        assert_eq!(app.version(), Version::new(0));
    }

    #[test]
    fn application_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = ApplicationId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn application_id_new_is_unique() {
        let id1 = ApplicationId::new();
        let id2 = ApplicationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn reconstitute_preserves_all_fields() {
        let state = ApplicationState::fake();

        let app = Application::reconstitute(ApplicationState {
            id: state.id.clone(),
            org_id: state.org_id.clone(),
            name: state.name.clone(),
            uid: state.uid.clone(),
            created_at: state.created_at,
            version: state.version,
        });

        assert_eq!(*app.id(), state.id);
        assert_eq!(*app.org_id(), state.org_id);
        assert_eq!(app.name(), state.name);
        assert_eq!(app.uid(), state.uid);
        assert_eq!(*app.created_at(), state.created_at);
        assert_eq!(app.version(), state.version);
    }
}
